mod hardcoded_special_transform;

use async_trait::async_trait;
use data_source_core::{DataSourceInterface, TxData};
use rumqttd::{Broker, Config, Notification};
use tracing::{debug, info};

use std::thread;

use crate::hardcoded_special_transform::SpecialEnvelope;

pub struct DataSourceMQTT {
    reserved: u32,
}

#[async_trait]
impl DataSourceInterface for DataSourceMQTT {
    async fn new_data_source(
        tx_new_data: TxData,
        config: &str,
    ) -> data_source_core::Result<DataSourceMQTT> {
        // TODO -- change loading this mqttd to be read in through the config passed in here
        let config = config::Config::builder()
            .add_source(config::File::with_name("rumqttd.toml"))
            .build()
            .unwrap();

        let config: Config = config.try_deserialize().unwrap();

        let mut broker = Broker::new(config);
        let (mut link_tx, mut link_rx) = broker.link("singlenode").unwrap();
        let broker_handle = thread::spawn(move || {
            broker.start().unwrap();
        });

        link_tx.subscribe("#").unwrap();

        let mut count = 0;
        let rx_handle = tokio::spawn(async move {
            loop {
                let notification = match link_rx.recv().unwrap() {
                    Some(v) => v,
                    None => continue,
                };

                match notification {
                    Notification::Forward(forward) => {
                        count += 1;
                        debug!(
                            "Topic = {:?}, Count = {}, Payload = {} bytes",
                            forward.publish.topic,
                            count,
                            forward.publish.payload.len()
                        );

                        // TODO - don't hardcode this
                        // Need to hardcode the transform here for quick dev of this and showcasing mqtt works
                        let data = SpecialEnvelope::new(
                            String::from_utf8(forward.publish.payload.to_vec()).unwrap(),
                        );
                        let data = match serde_json::to_string(&data) {
                            Ok(data) => data,
                            Err(err) => {
                                eprintln!("Could not convert envelope to string. [{}]", err);
                                return;
                            }
                        };
                        tx_new_data.send(data.as_bytes());
                    }
                    v => {
                        debug!("Notification: {v:?}");
                    }
                }
            }
        });

        Ok(DataSourceMQTT { reserved: 0 })
    }
}
