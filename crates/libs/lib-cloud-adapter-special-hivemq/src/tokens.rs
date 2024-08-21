use async_trait::async_trait;
use cloud_adapter_core::{DeliveryError, TokenDelivery};
use tokio::task::JoinHandle;
use tracing::trace;

use crate::SpecialHiveMQError;

pub struct DeliveryToken {
    pub msg_id: u32,
    pub future: JoinHandle<Result<u32, SpecialHiveMQError>>,
}

// Per the cloud-adapter contract, any adapter must implement TokenDelivery and CloudAdapter
#[async_trait]
impl TokenDelivery for DeliveryToken {
    async fn wait_for_ack(mut self) -> Result<u32, DeliveryError> {
        let res = self.future.await;
        let res = res
            .map_err(|err| DeliveryError {
                msg_id: self.msg_id,
                reason: err.to_string(),
            })?
            .map_err(|err| DeliveryError {
                msg_id: self.msg_id,
                reason: err.to_string(),
            });
        trace!("Ack Token result: [{:?}", res);
        res
    }
}
