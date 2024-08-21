use std::time::Duration;

use data_source_core::{DataSourceInterface, TxData};
use data_source_mqtt::DataSourceMQTT;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use tokio::{runtime::Handle, task, time};
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
pub async fn main() {
    let builder = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .with_line_number(false)
        .with_file(false)
        .with_thread_ids(false)
        .with_thread_names(false);

    builder
        .try_init()
        .expect("initialized subscriber succesfully");

    // Create the data source (this is what we are trying to test/play with)
    let (tx, mut rx) = TxData::new();
    let data_source = DataSourceMQTT::new_data_source(tx, "todo").await.unwrap();

    // Create a client, so we can see how the data source reacts
    let mut mqttoptions = MqttOptions::new("test-1", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    task::spawn(async move {
        requests(client).await;
        time::sleep(Duration::from_secs(3)).await;
    });

    task::spawn(async move {
        loop {
            let event = eventloop.poll().await;
            match &event {
                Ok(v) => {
                    //println!("Event = {v:?}");
                }
                Err(e) => {
                    //println!("Error = {e:?}");
                    return;
                }
            }
        }
    });

    // Ultimately, what we want to see is data received from the data_source_core::Rx
    // The mqtt client will send data, it will get picked up by the mqtt broker, and that broker will send it along with the TxData struct
    
    Handle::current().spawn(async move {
    //task::spawn(async move {
        loop {
            println!("listening for data");
            let data = rx.recv().await;
            println!("data: [{:?}]", data);
        }
    });

    tokio::time::sleep(Duration::from_secs(100)).await;
}
async fn requests(client: AsyncClient) {
    client
        .subscribe("hello/world", QoS::AtMostOnce)
        .await
        .unwrap();

    for i in 1..=10 {
        debug!("publishing some data");
        client
            .publish(
                "hello/world",
                QoS::ExactlyOnce,
                false,
                "i am some data!".as_bytes(),
            )
            .await
            .unwrap();

        time::sleep(Duration::from_secs(1)).await;
    }

    time::sleep(Duration::from_secs(120)).await;
}
