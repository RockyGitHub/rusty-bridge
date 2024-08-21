use serde::Deserialize;
use std::time::Duration;
use tracing::debug;

use async_trait::async_trait;
use data_source_core::{DataSourceInterface, TxData};

pub struct DataSourceDev {
    pub name: String,
    pub msg_generator: tokio::task::JoinHandle<()>,
    //pub subscriptions: Vec<JoinHandle<()>>,
    //pub rx: Receiver<MsgBusData>,
}

#[derive(Deserialize)]
struct Config {
    _ingress_rate: u32,
}

// TODO - at some point make the dev data source a TUI
#[async_trait]
impl DataSourceInterface for DataSourceDev {
    #[allow(warnings)]
    async fn new_data_source(
        tx_new_data: TxData,
        config: &str,
    ) -> data_source_core::Result<DataSourceDev> {
        //let config: Config =
        //serde_json::from_str(config).map_err(|err| Error::Initialize(err.to_string()))?;

        // Setting up a fake source of messages
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        // Start a thread that acts as a source of generated data. This data would normally come from some place like bacnet connector, rabbitmq, etc
        let handle = tokio::task::spawn(async move {
            loop {
                //let mut rng = rand::thread_rng();
                let payload = EXAMPLE_SPECIAL_MSG.to_string();
                debug!("Payload generated: [{}]", payload);

                tx.send(payload).await;
                tokio::time::sleep(Duration::from_millis(5000)).await;
            }
        });

        // Publish the data
        //for topic in config_data.topics { }
        tokio::task::spawn(async move {
            loop {
                // rx is just the idea of picking up a message from source. like zmq, rabbitmq, http, etc
                match rx.recv().await {
                    Some(msg) => {
                        // Convert to MsgBusData
                        let msg = msg.as_bytes();
                        tx_new_data.send(msg);
                    }
                    None => break,
                }
            }
        });

        Ok(DataSourceDev {
            name: "Message Bus Development".to_string(),
            msg_generator: handle,
        })
    }
}

pub const EXAMPLE_SPECIAL_MSG: &str = r#"
{
  "data": {
  }
}
"#;
