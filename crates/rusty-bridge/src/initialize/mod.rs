use cloud_adapter_core::CloudAdapterTrait;
use data_source::new_data_source;
use data_source_core::{DataSourceInterface, RxData, TxData};
use edge_reporter::EdgeReporter;
use mini_config::new_config;
use mini_config_core::ConfigData;
use mini_config_core::MiniConfigInterface;
use msg_transform_core::MsgTransform;
use msg_transforms::init_msg_transformer;
use tokio_util::sync::CancellationToken;

use crate::{data_server::DataServerHandle, error::RustyBridgeError, persistence::init_persistence};

use self::signals::register_shutdown_signals;

mod signals;

// throttle_rate * token_timeout will give the potential in-flight queue size for the ack queue
// PUBLISH_CHANNEL_CAPACITY should be set to some reasonable amount.  The only way to find out is to benchmark how fast it can move messages from message bus to the ack channel
const PUBLISH_CHANNEL_CAPACITY: usize = 1000;

pub type Result<T> = std::result::Result<T, RustyBridgeError>;

#[derive(Debug)]
pub enum InitError {
    MessageBus(String),
    Configuration(String),
    CloudAdapter(String),
    DataServer(String),
    EdgeReporter(String),
}

/// Initialization of dependencies. This will create the concrete types of `CloudAdapter`, `MessageBusInterface`, `MsgTransform`.
/// These generic impl's are made concrete by specifying feature flags. For example, `--features="msg-bus/dev"` will build the developer build
///
pub async fn initialize() -> Result<(
    impl CloudAdapterTrait,
    impl DataSourceInterface,
    impl MsgTransform,
    ConfigData,
    RxData,
    DataServerHandle,
    CancellationToken,
)> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let shutdown_token = CancellationToken::new();
    let signal_handle = register_shutdown_signals(shutdown_token.clone())?;

    let config_fetcher = new_config().map_err(|err| InitError::Configuration(err.to_string()))?;

    // If the data-server feature is enabled, startup the data-server that will serve up metrics data to any consuming client
    let metrics_handle = crate::data_server::init_data_server(shutdown_token.clone()).await?;

    // Get ConfigurationData -- this comes from an interaction on the message bus
    let config_data = config_fetcher
        .get_config()
        .await
        .map_err(|err| InitError::Configuration(err.to_string()))?;

    // Create the publish channels -- that are used for getting a message from the message-bus to the msg-engine
    // TODO - pass metrics handle here so that data can be submitted at each .send() call
    let (tx_new_msg, rx_new_msg) = TxData::new();

    // Initialize DataSource -- This is where the connector receives messages from. Additional data sources should use the same tx..
    let data_source = new_data_source(tx_new_msg, &config_data.data_source)
        .await
        .map_err(|err| InitError::MessageBus(err.to_string()))?;

    // Create persistence
    let persistence = init_persistence();

    // Create the CloudAdapter -- the logic that will transform and publish a message to the cloud
    // TODO create all connectors here, eventually I want to support multiple connectors but not till I really know the flow of everything yet
    let adapter = cloud_adapter::new("special-hivemq", &config_data.north_adapters)
        .map_err(|err| InitError::CloudAdapter(format!("{:?}", err)))?;

    // Create the Transform object -- this transform, transforms the msg-bus message to a format the cloud server is expecting
    let transform = init_msg_transformer();

    // The edge reporter reports this edge to the cloud. It helps track all JCI edges in one location
    let edge_reporter = EdgeReporter::new(&config_data.edge_reporter)
        .map_err(|err| InitError::EdgeReporter(err.to_string()))?;
    let reporter_handle = edge_reporter.start_reporting();

    Ok((
        adapter,
        data_source,
        transform,
        config_data,
        rx_new_msg,
        metrics_handle,
        shutdown_token,
    ))
}

//fn convert_credentials_type(
//credentials: data_source_core::Credentials,
//) -> cloud_adapter_core::Credentials {
//cloud_adapter_core::Credentials {
//username: credentials.username,
//password: credentials.password,
//custom: credentials.custom,
//}
//}

impl From<InitError> for RustyBridgeError {
    fn from(value: InitError) -> Self {
        RustyBridgeError::Initialization(format!("{:?}", value))
    }
}
