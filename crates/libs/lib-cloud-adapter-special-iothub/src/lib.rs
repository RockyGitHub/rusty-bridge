//use std::future::Future;

//use async_trait::async_trait;
//use cloud_adapter_core::{
//CloudAdapterTrait, ConnectionError, TokenConnection, TokenDelivery, TokenDisconnect,
//};
//use msg_bus_core::MsgBusData;
//use tokio::{
//sync::broadcast,
//time::{sleep, Duration, Sleep},
//};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    _reserved: u32,
}

//pub struct SpecialIoTHub {}

//pub struct DeliverContext {
//id: String,
//future: Option<Sleep>,
//}

//pub struct ConnectToken {
//future: Option<Sleep>,
//}

//impl SpecialIoTHub {
//pub fn new(tx_conn_status: broadcast::Sender<bool>) -> Self {
//Self {}
//}
//}

//#[async_trait]
//impl TokenDelivery for DeliverContext {
//async fn wait_for_ack(mut self) -> Result<(), cloud_adapter_core::DeliveryError> {
//self.future.take().unwrap().await;
//Ok(())
//}
//}

//#[async_trait]
//impl TokenConnection for ConnectToken {
//async fn wait_for_connect(mut self) -> Result<(), cloud_adapter_core::ConnectionError> {
//self.future.take().unwrap().await;
//Ok(())
//}
//}

////#[async_trait]
//impl CloudAdapterTrait for SpecialIoTHub {
////fn publish(&self, msg: MsgBusData) -> Box<dyn TokenDelivery + Send> {
//fn publish(&mut self, msg: MsgBusData) -> impl TokenDelivery + 'static {
////self.client.publish();
//println!("iothub publishing [{:?}]!", msg);
//let token = DeliverContext {
//id: "iothub".to_string(),
//future: Some(sleep(Duration::from_millis(100))),
//};
//token
////tokio::time::sleep(tokio::time::Duration::from_millis(100))
//}

//async fn connect(&mut self) -> impl cloud_adapter_core::TokenConnection + Send + 'static {
////Box::new(ConnectToken {
////future: Some(sleep(Duration::from_millis(500))),
////})
//ConnectToken {
//future: Some(sleep(Duration::from_millis(500))),
//}
//}

//fn disconnect(
//&mut self,
//) -> Result<impl Future<Output = Result<(), ConnectionError>>, ConnectionError> {
//// First attempt at this, it doesn't do anything yet. This is cool!
////todo!();
//let (tx, rx) = tokio::sync::oneshot::channel::<()>();
//let token = TokenDisconnect {
//future: async move {
//rx.await;
//Ok(())
//},
//};
//Ok(token)
//}
//}
