mod data_source;
mod edge_reporter;
mod metrics_server;
mod north_adapters;
mod persistence;

use async_trait::async_trait;
use data_source::get_data_source;
use edge_reporter::get_edge_reporter;
use metrics_server::get_metrics_server;
use mini_config_core::{ConfigData, Error, MiniConfigInterface, Result};
use north_adapters::get_north_adapters;
use persistence::get_persistence;
use tracing::trace;

const SERVER_TYPE: &str = "connector";

pub struct MiniConfigSpecial {
    //config_client: config::ConfigClient,
}

impl MiniConfigSpecial {
    pub fn new() -> Result<MiniConfigSpecial> {
        let fetcher = MiniConfigSpecial {};

        Ok(fetcher)
    }
}

#[async_trait]
impl MiniConfigInterface for MiniConfigSpecial {
    async fn get_config(&self) -> Result<ConfigData> {
        // Unfortunately the foghorn ConfigClient doesn't support being initialized under an existing runtime at the moment.. So everything needs to run in its own thread
        let config_data = std::thread::spawn(|| {
            let manager_host = "192.168.56.102";
            let manager_port = 8001;
            let runtime =
                tokio::runtime::Runtime::new().map_err(|err| Error::GetConfig(err.to_string()))?;

            let data_source = get_data_source("tmp".to_string())?;
            let data_source = serde_json::to_string(&data_source)
                .map_err(|err| Error::GetConfig(err.to_string()))?;

            // TODO - only 1 adapter is supported at the moment
            let north_adapters = get_north_adapters("tmp".to_string())?;
            let north_adapters = serde_json::to_string(&north_adapters)
                .map_err(|err| Error::GetConfig(err.to_string()))?;

            let edge_reporter = get_edge_reporter("tmp".to_string())?;
            let edge_reporter = serde_json::to_string(&edge_reporter)
                .map_err(|err| Error::GetConfig(err.to_string()))?;

            let metrics_server = get_metrics_server("tmp".to_string())?;
            let metrics_server = serde_json::to_string(&metrics_server)
                .map_err(|err| Error::GetConfig(err.to_string()))?;

            let persistence = get_persistence("tmp".to_string())?;
            let persistence = serde_json::to_string(&persistence)
                .map_err(|err| Error::GetConfig(err.to_string()))?;

            let config_data = ConfigData {
                data_source,
                north_adapters: north_adapters,
                edge_reporter,
                metrics_server,
                persistence,
                file_uploads: None,
            };
            trace!("Loaded config: [{:?}]", config_data);

            Ok(config_data)
        })
        .join()
        .map_err(|err| Error::GetConfig(format!("{:?}", err)))??;

        Ok(config_data)
    }
}
