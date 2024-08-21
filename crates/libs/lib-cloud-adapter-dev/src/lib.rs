use std::future::Future;

use async_trait::async_trait;
use cloud_adapter_core::{
    CloudAdapterTrait, ConnectionError, ConnectionLost, TokenConnection, TokenDelivery,
    TokenDisconnect,
};
use data_source_core::MsgBusData;
use tokio::{
    spawn,
    sync::{broadcast, watch},
    time::{sleep, Duration, Sleep},
};
use tracing::debug;

pub struct Dev {
    tx_conn_status: broadcast::Sender<bool>,
}

pub struct DeliverContext {
    id: String,
    future: Option<Sleep>,
}

pub struct ConnectToken {
    future: Option<Sleep>,
}

impl Dev {
    pub fn new(tx_conn_status: broadcast::Sender<bool>) -> Self {
        Self { tx_conn_status }
    }
}

#[async_trait]
impl TokenDelivery for DeliverContext {
    async fn wait_for_ack(mut self) -> Result<u32, cloud_adapter_core::DeliveryError> {
        let msg_id = self.future.take().unwrap().await;
        let msg_id = 0;
        Ok(msg_id)
    }
}

//#[async_trait]
//impl TokenConnection for ConnectToken {
//async fn wait_for_connect(mut self) -> Result<(), cloud_adapter_core::ConnectionError> {
//self.future.take().unwrap().await;
//Ok(())
//}
//}

//#[async_trait]
impl CloudAdapterTrait for Dev {
    //fn publish(&self, msg: MsgBusData) -> Box<dyn TokenDelivery + Send> {
    fn publish(&mut self, msg: MsgBusData) -> impl TokenDelivery + 'static {
        //self.client.publish();
        let payload = std::str::from_utf8(&msg.payload).unwrap();
        debug!(
            "dev publishing: id: [{}] retry count: [{}] payload: [{}]!",
            msg.id, msg.retry_count, payload
        );

        DeliverContext {
            id: "dev".to_string(),
            future: Some(sleep(Duration::from_millis(100))),
        }
        //tokio::time::sleep(tokio::time::Duration::from_millis(100))
    }

    async fn connect(
        &mut self,
    ) -> Result<
        TokenConnection<
            impl Future<Output = Result<watch::Receiver<ConnectionLost>, ConnectionError>> + 'static,
        >,
        ConnectionError,
    > {
        let (tx_conn_lost, mut rx_conn_lost) =
            watch::channel::<ConnectionLost>(ConnectionLost::Uncategorized("init".to_string()));
        rx_conn_lost.borrow_and_update();
        let token = TokenConnection {
            future: async move {
                sleep(Duration::from_millis(500)).await;
                Ok(rx_conn_lost)
            },
        };

        // Spawn a task to simulate a disconnection event
        spawn(async move {
            sleep(Duration::from_secs(10)).await;
            let _ = tx_conn_lost.send(ConnectionLost::Timeout);
        });

        Ok(token)
    }

    fn disconnect(
        &mut self,
    ) -> Result<impl Future<Output = Result<(), ConnectionError>> + 'static, ConnectionError> {
        // First attempt at this, it doesn't do anything yet. This is cool!
        //todo!();
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        let token = TokenDisconnect {
            future: async move {
                let _ = rx.await;
                Ok(())
            },
        };
        Ok(token)
    }
}
