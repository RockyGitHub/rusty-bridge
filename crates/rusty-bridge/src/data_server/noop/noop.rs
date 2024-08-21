use crate::initialize::InitError;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub struct DataServerHandle {}

pub async fn init_data_server(shutdown: CancellationToken) -> Result<DataServerHandle, InitError> {
    info!("Data server was not compiled into this service, it will not be accessible");

    Ok(DataServerHandle {})
}

impl DataServerHandle {
    pub fn event_connection(&self, connected: bool) {}
    pub fn event_rx_data(&self, id: u32) {}
    pub fn event_pub_data(&self, id: u32) {}
    pub fn event_pub_ack(&self, id: u32, success: bool) {}
}
