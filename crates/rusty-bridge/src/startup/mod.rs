//use data_source_core::{ConfigData, DataSourceInterface, MsgBusData};
//use msg_transform_core::MsgTransform;
//use tokio::sync::mpsc;
//use tracing::{error, info};

// Performs startup logic, effectively a continuation of initialization, but not including the creation of objects
//pub async fn startup(
//msg_bus: &impl DataSourceInterface,
//config_data: ConfigData,
//tx_new_msg: mpsc::Sender<MsgBusData>,
//transform: impl MsgTransform,
//) {
//// Subscribe to topics
//// TODO - instead of passing a callback that will require a channel, how about returning a Stream object instead?
//for topic in config_data.subscription_topics {
//let north_tx_channel = north_tx_channel.clone();
//if let Err(err) = msg_bus
//.subscribe_with_callback(&topic, move |data: MsgBusData| {
//// Transform the data to the cloud protocol
//// Should this be performed in the cloud adapter?
//// Expect &[u8] in, &[u8] out

//// Persistence occurs later, because we only persist if on nack...
//// This means we have to allow for [nack timeout] worth of messages to be lost in the case of power loss

//if let Err(err) = north_tx_channel.try_send(data) {
//// TODO - this is a critical error and a soft restart should likely be performed. This could result in the loss of data, but such is the way when
//// there's a critical error..
//error!(
//"Message could not be sent through northbound channel, it will be dropped. Error: [{}]", err
//);
//}
//})
//.await
//{
//error!(
//"Could not subscribe to topic [{}], error: [{:?}]",
//topic, err
//);
//}
//info!("Subscribed to topic: [{}]", topic);
//}
//}
