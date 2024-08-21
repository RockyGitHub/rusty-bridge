//! # Special HiveMQ Adapter
//!
//! todo()! description
//!
mod error;
mod event_loop;
mod tokens;
use std::future::Future;
use std::sync::Arc;

use crate::event_loop::spawn_looper;
pub use crate::tokens::DeliveryToken;
pub use error::SpecialHiveMQError;

use cloud_adapter_core::{
    CloudAdapterTrait, ConnectionError, ConnectionLost, TokenDelivery, TokenDisconnect,
};
use cloud_adapter_core::{Error, TokenConnection};
use data_source_core::MsgBusData;
use rumqttc::{
    AsyncClient, ConnectReturnCode, EventLoop, MqttOptions, TlsConfiguration, Transport,
};
use serde::{Deserialize, Serialize};
use special_ana::SpecialAnA;
use special_hivemq_transform::TransformSpecialHiveMQ;
use tokio::sync::watch;
use tokio::{
    spawn,
    sync::{broadcast, mpsc, oneshot},
    task::JoinHandle,
    time::Duration,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, trace};

pub struct SpecialHiveMQ {
    client: AsyncClient,
    credentials: Credentials,

    // because publish() doesn't return the pktid...
    /// Sends a oneshot::Sender that will return the msg.id, as well as the msg.id itself
    /// This is needed because client.publish() does not return an id
    tx_ack_channel: tokio::sync::mpsc::Sender<(u32, oneshot::Sender<u32>)>,
    tx_connect_ack: broadcast::Sender<ConnectReturnCode>,
    rx_conn_lost: watch::Receiver<ConnectionLost>,
    /// Blocks or unblocks the event_loop because event_loop will always reconnect
    tx_run: watch::Sender<bool>,
    // poller handling
    event_handle: JoinHandle<EventLoop>,
    /// Transforms a &[u8] to a sparkplugb message
    transform: TransformSpecialHiveMQ,
    /// To shutdown the event_loop
    shutdown: CancellationToken,
}

#[derive(Clone)]
struct Credentials {
    username: String,
    topic_data: String,
    topic_cmd: String,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub ana_endpoint: String,
    pub mqtt_endpoint: String,
}

impl SpecialHiveMQ {
    // TODO get rid of this tx_conn_status thing.. I don't think it's a good idea
    pub fn new(config: &str) -> Result<Self, Error> {
        let config = serde_json::from_str::<Config>(config)
            .map_err(|err| SpecialHiveMQError::Init(err.to_string()))?;

        let Config {
            username,
            password,
            ana_endpoint,
            mqtt_endpoint,
        } = config;
        // Parse the custom field of the Credentials struct to get the ana endpoint and hivemq endpoint
        let ana = SpecialAnA::new_mqtt(&ana_endpoint, username.clone(), password, true)
            .map_err(|err| SpecialHiveMQError::Init(err.to_string()))?;

        // TODO improve this, perhaps with a regex?
        let username_split: Vec<&str> = username.split('@').collect();
        let edge_node_id = username_split[0];
        let group_id = username_split[1].to_string();
        let group_id = group_id.trim_end_matches(".com");

        // Setup TLS for WSS by obtaining the certificates
        let mut root_cert_store = rumqttc::tokio_rustls::rustls::RootCertStore::empty();
        root_cert_store.add_parsable_certificates(
            rustls_native_certs::load_native_certs()
                .map_err(|err| SpecialHiveMQError::Init(err.to_string()))?,
        );
        let client_config = rumqttc::tokio_rustls::rustls::ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();

        let mut mqtt_options = MqttOptions::new(group_id, mqtt_endpoint, 443);

        mqtt_options.set_clean_session(true);
        mqtt_options.set_inflight(2000);
        mqtt_options.set_keep_alive(Duration::from_millis(30_000));
        mqtt_options.set_manual_acks(false);
        //mqtt_options.set_pending_throttle(Duration::from_millis(1));
        mqtt_options.set_request_channel_capacity(10);
        //mqtt_options.set_transport(Transport::Wss(TlsConfiguration::default()));
        mqtt_options.set_transport(Transport::Wss(TlsConfiguration::Rustls(Arc::new(
            client_config,
        ))));
        // Do not set LastWill unless it is supported in HiveMQ first. Otherwise connetions will result in "notauthorized"
        //let last_will = LastWill::new("last_will_topic", "x.x", rumqttc::QoS::ExactlyOnce, true);
        //mqtt_options.set_last_will(last_will);

        let topic_cmd = format!("spBv1.0/{}/NCMD/{}", group_id, edge_node_id);
        let topic_data = format!("spBv1.0/{}/NDATA/{}", group_id, edge_node_id);
        trace!(
            "Command topic: [{}]. Data topic: [{}]",
            topic_cmd,
            topic_data
        );

        let (tx_ack_channel, rx_ack_channel) = mpsc::channel(10);
        let (tx_connect_ack, _) = tokio::sync::broadcast::channel(1);
        let (tx_run, rx_run) = watch::channel(false);
        tx_run.send_replace(false);

        let (client, event_loop) = AsyncClient::new(mqtt_options, 10);
        let (tx_conn_lost, mut rx_conn_lost) =
            watch::channel(ConnectionLost::Uncategorized("init".to_string()));
        let _ = rx_conn_lost.borrow_and_update(); // TODO - this might not be needed

        let credentials = Credentials {
            username,
            topic_data,
            topic_cmd,
        };

        let shutdown = CancellationToken::new();
        let event_handle = spawn_looper(
            event_loop,
            rx_run,
            tx_connect_ack.clone(),
            rx_ack_channel,
            tx_conn_lost,
            credentials.clone(),
            ana,
            client.clone(),
            shutdown.child_token(),
        );

        Ok(Self {
            client,
            credentials: credentials,
            tx_connect_ack,
            tx_ack_channel,
            rx_conn_lost,
            tx_run,
            transform: TransformSpecialHiveMQ::new(),
            event_handle,
            shutdown,
        })
    }
}

impl CloudAdapterTrait for SpecialHiveMQ {
    fn publish(&mut self, msg: MsgBusData) -> impl TokenDelivery + 'static {
        //debug!("Publishing [{:?}]!", msg);
        trace!("Publishing [{}]", msg.id);
        let id = msg.id;

        // Instead of spawning a task and cloning, I could just use try_send?
        let client_clone = self.client.clone();
        let tx_publish_reqst_clone = self.tx_ack_channel.clone();

        let msg = self.transform.transform(msg.payload);

        let topic_data = self.credentials.topic_data.clone();

        // If publish before token creation. I like this better I think, but if client.try_publish fails, what to do about the mismatch in tx_ack_channel then?
        //let (tx_ack, rx_ack) = oneshot::channel();
        //self.tx_ack_channel.try_send((id.into(), tx_ack)).unwrap();
        //self.client.try_publish(topic_data, rumqttc::QoS::AtLeastOnce, false, msg).unwrap();

        let handle = spawn(async move {
            // TODO - logic here is a little weird because publish doesn't occur until the token is awaited. but it's nice because it prevents any blocking in main_loop. maybe try try_send instead?
            // Send/Queue up a oneshot channel for the event_poller to use to complete this task
            let (tx_ack, rx_ack) = oneshot::channel();
            tx_publish_reqst_clone
                .send((id.into(), tx_ack))
                .await
                .unwrap();

            // Then publish the message. Don't publish before sending the oneshot, or else a race condition may occur
            client_clone
                .publish(topic_data, rumqttc::QoS::AtLeastOnce, false, msg)
                .await
                .unwrap();

            rx_ack
                .await
                .map_err(|err| SpecialHiveMQError::PubAck(format!("no ack: [{}]", err)))
        });

        let token = DeliveryToken {
            future: handle,
            msg_id: id,
        };

        token
    }

    // TODO - made this async.. is that a problem?
    async fn connect(
        &mut self,
    ) -> Result<
        TokenConnection<
            impl Future<Output = Result<watch::Receiver<ConnectionLost>, ConnectionError>> + 'static,
        >,
        ConnectionError,
    > {
        debug!("Connecting to HiveMQ");
        self.tx_run.send_replace(true);

        let mut rx_conn_result = self.tx_connect_ack.subscribe();
        let client_clone = self.client.clone();
        let rx_conn_lost_clone = self.rx_conn_lost.clone();
        let topic_cmd = self.credentials.topic_cmd.clone();
        let token = TokenConnection {
            future: async move {
                let code = rx_conn_result
                    .recv()
                    .await
                    .map_err(|err| ConnectionError::Failure(err.to_string()))?;

                client_clone
                    .subscribe(topic_cmd, rumqttc::QoS::AtLeastOnce)
                    .await
                    .unwrap();

                if code != ConnectReturnCode::Success {
                    return Err(ConnectionError::Failure(format!(
                        "{:?} code: [{}]",
                        code, code as u8
                    )));
                }

                Ok(rx_conn_lost_clone)
            },
        };

        Ok(token)
    }

    // TODO - should handle a confirmation of disconnect
    fn disconnect(
        &mut self,
    ) -> Result<impl Future<Output = Result<(), ConnectionError>> + 'static, ConnectionError> {
        self.client
            .try_disconnect()
            .map_err(|err| ConnectionError::Failure(format!("{}", err)))?;

        // TODO - need to set self.tx_run.send_replace(false) but it needs to wait for the disconnect() to go through first..
        //self.rx_conn_lost.
        //self.tx_run.send_replace(false);
        // channel.subscibe() to something to indicate a succesful disconnect and pass it to token
        let token = TokenDisconnect {
            future: async move { Ok(()) },
        };
        Ok(token)
    }
}

impl Drop for SpecialHiveMQ {
    fn drop(&mut self) {
        // TODO await the event_handle to close
        trace!("Dropping SpecialHiveMQ, waiting for event_handle..");
        let _ = self.client.try_disconnect();
        self.shutdown.cancel();
        self.event_handle.is_finished();
    }
}
