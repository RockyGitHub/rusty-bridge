use std::env;

use async_trait::async_trait;
use mini_config_core::{ConfigData, Error, MiniConfigInterface, Result};
use serde::{Deserialize, Serialize};
use tracing::warn;

const PATH_TO_CONFIG: &str = "PATH_TO_MINIEDGE_DEV_CONFIG";

pub struct MiniConfigDev {
    _res: i32,
}

#[derive(Serialize, Deserialize)]
struct DataSourceSpecial {
    pub_endpoint: String,
    sub_endpoint: String,
    highwater_mark: u32,
    topics: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct DataSourceHttpRest {
    bind_address: String,
}

// For special hivemq
#[derive(Deserialize, Serialize)]
struct NorthAdapter {
    username: String,
    password: String,
    ana_endpoint: String,
    mqtt_endpoint: String,
}

#[derive(Deserialize, Serialize)]
struct EdgeReporter {
    endpoint: String,
    system_name: String,
    interval_s: u32,
}

#[derive(Deserialize, Serialize)]
struct MetricsServer {
    enabled: bool,
}

#[derive(Deserialize, Serialize)]
struct Persistence {
    enabled: bool,
    highwater_mb: u32,
}

#[derive(Deserialize, Serialize)]
struct FileUploader {
    _res: u32,
}

#[derive(Deserialize)]
struct TomlData {
    data_source: DataSourceHttpRest,
    north_adapter: NorthAdapter,
    edge_reporter: EdgeReporter,
    metrics_server: MetricsServer,
    persistence: Persistence,
    file_uploader: Option<FileUploader>,
}

impl MiniConfigDev {
    pub fn new() -> Result<MiniConfigDev> {
        let fetcher = MiniConfigDev { _res: 0 };

        Ok(fetcher)
    }
}

#[async_trait]
impl MiniConfigInterface for MiniConfigDev {
    async fn get_config(&self) -> Result<ConfigData> {
        let config_path = match env::var(PATH_TO_CONFIG) {
            Ok(path) => path,
            Err(err) => {
                warn!(
                    "Could not find env variable for configuration file. [{}]. [{}]. Will search for 'config.toml' in the current directory",
                    PATH_TO_CONFIG, err
                );
                "config.toml".to_string()
            }
        };
        //let path = "./crates/libs/lib-mini-config-dev/config.toml";
        let config = std::fs::read_to_string(&config_path)
            .map_err(|err| Error::GetConfig(err.to_string()))?;
        let data = toml::from_str::<TomlData>(&config).unwrap();

        let data_source = serde_json::to_string(&data.data_source)
            .map_err(|err| Error::GetConfig(err.to_string()))?;
        let north_adapters = serde_json::to_string(&data.north_adapter)
            .map_err(|err| Error::GetConfig(err.to_string()))?;
        let edge_reporter = serde_json::to_string(&data.edge_reporter)
            .map_err(|err| Error::GetConfig(err.to_string()))?;
        let metrics_server = serde_json::to_string(&data.metrics_server)
            .map_err(|err| Error::GetConfig(err.to_string()))?;
        let persistence = serde_json::to_string(&data.persistence)
            .map_err(|err| Error::GetConfig(err.to_string()))?;
        if let Some(_file_uploader) = data.file_uploader {
            todo!()
        }

        // TODO - why am I converting to a similar and mostly same type
        let config_data = ConfigData {
            data_source,
            north_adapters,
            edge_reporter,
            metrics_server,
            persistence,
            file_uploads: None,
        };

        Ok(config_data)
    }
}
