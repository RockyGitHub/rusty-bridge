use async_trait::async_trait;

mod error;
pub use error::Error;

pub type Result<T> = core::result::Result<T, error::Error>;

#[derive(Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub custom: Option<String>,
}

//pub struct ConfigData {
//pub system_name: String,
//pub reporter_endpoint: String,
//pub reporter_interval_s: u32,
///// The cloud adapters to use (only supporting 1 right now)
//pub adapters: Vec<String>,
//pub credentials: Credentials,
//// TODO - how to make data_source a concrete type?
//pub data_source: String,
//}

#[derive(Debug)]
pub struct ConfigData {
    pub data_source: String,
    pub north_adapters: String,
    pub edge_reporter: String,
    pub metrics_server: String,
    pub persistence: String,
    pub file_uploads: Option<String>,
}

#[async_trait]
pub trait MiniConfigInterface {
    async fn get_config(&self) -> Result<ConfigData>;
}
