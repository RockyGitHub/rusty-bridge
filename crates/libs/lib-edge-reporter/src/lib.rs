mod dynamic_metrics;
mod static_metrics;

use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{spawn, task::JoinHandle};
use tracing::{debug, error, info, warn};

use self::{dynamic_metrics::DynamicSystemMetrics, static_metrics::StaticSystemMetrics};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to initialize due to [{0}]")]
    Initialization(String),
}

pub struct EdgeReporter {
    config: ReporterConfig,
    static_metrics: StaticSystemMetrics,
    dynamic_metrics: DynamicSystemMetrics,
}

#[derive(Serialize, Deserialize)]
pub struct ReporterConfig {
    pub endpoint: String, // todo use uri or url insetad
    pub system_name: String,
    pub interval_s: u32,
}

impl EdgeReporter {
    pub fn new(
        config: &str,
        //system_name: String,
        //reporter_endpoint: String,
        //refresh_rate: u32,
    ) -> Result<EdgeReporter> {
        let config = serde_json::from_str::<ReporterConfig>(config).unwrap();
        //system_name = "development system"             # This should be set to a team's chosen choice and is used by the Edge Reporter
        //endpoint = "http://127.0.0.1:8080/edge_report"
        //interval_s = 60

        let static_metrics = StaticSystemMetrics::new(config.system_name.clone())?;
        let dynamic_metrics = DynamicSystemMetrics::new(static_metrics.machine_id().to_owned())?;

        let reporter = EdgeReporter {
            static_metrics,
            dynamic_metrics,
            config,
        };

        Ok(reporter)
    }

    pub fn start_reporting(self) -> JoinHandle<()> {
        spawn(async move {
            if self.config.interval_s == 0 {
                info!("Edge reporter will not run due to setting interval_s to 0");
                return;
            }
            info!("Edge reporter starting");
            let client = reqwest::Client::new();
            let endpoint = self.config.endpoint;
            let interval_s = self.config.interval_s;
            let static_data = self.static_metrics;
            let mut dynamic_data = self.dynamic_metrics;

            // Start with sending the static data
            if let Err(err) = send_static_data(&client, &static_data, &endpoint, interval_s).await {
                error!("Could not send static data to edge listing, this may prevent this edge from appearing. [{}]", err);
            }
            loop {
                if let Err(err) = send_dynamic_data(&client, &mut dynamic_data, &endpoint).await {
                    warn!("Could not send dynamic to JCI listing. [{}]", err);
                };
                tokio::time::sleep(Duration::from_secs(interval_s.into())).await;
            }
        })
    }
}

async fn send_dynamic_data(
    client: &reqwest::Client,
    dynamic_data: &mut DynamicSystemMetrics,
    endpoint: &str,
) -> Result<()> {
    let endpoint = format!("{}/{}", endpoint, "dynamic");
    dynamic_data.update();
    //let dynamic_data = serde_json::to_string(dynamic_data)
    //.map_err(|err| RustyBridgeError::EdgeReporter(err.to_string()))?;

    match client.post(endpoint).json(&dynamic_data).send().await {
        Ok(response) => {
            let status = response.status();
            let text = response.text().await;
            if let Err(err) = &text {
                error!("Response from JCI edge listing had an issue: [{}]", err);
            };
            debug!(
                "Dynamic data sent succesfully, status: [{:?}], text: [{:?}]",
                status, text
            );
        }
        Err(_) => todo!(),
    }

    Ok(())
}

async fn send_static_data(
    client: &reqwest::Client,
    static_data: &StaticSystemMetrics,
    endpoint: &str,
    interval_s: u32,
) -> Result<()> {
    let endpoint = format!("{}/{}", endpoint, "static");
    //let static_data = serde_json::to_string(static_data)
    //.map_err(|err| RustyBridgeError::EdgeReporter(err.to_string()))?;
    // Retry until succesful
    loop {
        match client.post(&endpoint).json(&static_data).send().await {
            Ok(response) => {
                let status = response.status();
                let text = response.text().await;
                // If the response returns an err, report it but proceed anyway
                if let Err(err) = &text {
                    error!("Response from JCI edge listing had an issue: [{}]", err);
                };
                debug!(
                    "Static data sent succesfully, status: [{:?}], text: [{:?}]",
                    status, text
                );
                break;
            }
            Err(err) => warn!(
                "Could not POST static to JCI, will try again in [{}] seconds. [{:?}]",
                interval_s, err
            ),
        }
        tokio::time::sleep(Duration::from_secs(interval_s.into())).await;
    }

    Ok(())
}
