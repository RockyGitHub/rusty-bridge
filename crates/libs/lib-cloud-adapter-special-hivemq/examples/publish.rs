use std::time::Duration;

use cloud_adapter_core::CloudAdapterTrait;
use cloud_adapter_core::TokenDelivery;
use data_source_core::MsgBusDataFactory;
use special_hivemq::Config;
use tokio::spawn;
use tracing::info;

#[tokio::main]
async fn main() {
    // Init tracing collector so we can view logs from the adapter
    tracing_subscriber::fmt::init();

    // Initialize the adapter
    let config = Config {
        username: "username".to_string(),
        password: "password".to_string(),
        ana_endpoint: "https://www.myanawebsite.com/api/v1/hivemq".to_string(),
        mqtt_endpoint: "wss://mqtt.placeholder.com:443/mqtt".to_string(),
    };
    // TODO - trying to figure out how to maintain concrete types for rusty edge to pas
    let config = serde_json::to_string(&config).unwrap();
    let mut adapter = special_hivemq::SpecialHiveMQ::new(&config).unwrap();

    // Connect to cloud
    let token = adapter.connect().await.unwrap();

    // The adapter returns a token, you can use this to verify the success or failure of a connection attempt
    let res = token.await;
    info!("connect token result: {:?}", res);

    // Publish some data
    for _ in 0..2 {
        let data = MsgBusDataFactory::new().msg(EXAMPLE_SPECIAL_MSG.as_bytes());
        let token = adapter.publish(data);
        // TODO - I should be able to pass the token into a task
        spawn(async move {
            let res = token.wait_for_ack().await;
            info!("publish result: {:?}", res);
        });
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }

    // Disconnect cleanly
    let token = adapter.disconnect().unwrap();
    //let res = token.await;
    spawn(async move {
        let res = token.await;
        info!("Disconnect result: {:?}", res);
    })
    .await
    .unwrap();

    tokio::time::sleep(Duration::from_secs(3000)).await;
}

pub const EXAMPLE_SPECIAL_MSG: &str = r#"
{
  "data": {
    "somedata": "somedata"
  }
}
"#;

pub const EXAMPLE_SPARKPLUG: &'static [u8] = &[0, 0]; // TODO get example byte based packet
