///
/// #### Notes ####
///
/// should just the contract peices live here and the "create_message_bus" stuff live in the connector?
///
///
///
///
///
///
///
use data_source_core::DataSourceInterface;
use data_source_core::TxData;

#[cfg(all(feature = "data-source-dev", feature = "special"))]
compile_error!("only one data-source feature can be chosen at a time");

#[cfg(all(feature = "data-source-dev", feature = "special"))]
compile_error!("only one data-source feature can be chosen at a time");
//

#[cfg(feature = "mqtt")]
pub async fn new_data_source(
    tx_new_data: TxData,
    config: &str,
) -> data_source_core::Result<impl DataSourceInterface> {
    use data_source_mqtt::DataSourceMQTT;

    DataSourceMQTT::new_data_source(tx_new_data, config).await
}

#[cfg(feature = "http-rest")]
pub async fn new_data_source(
    tx_new_data: TxData,
    config: &str,
) -> data_source_core::Result<impl DataSourceInterface> {
    use data_source_http_rest::DataSourceHttpRest;

    DataSourceHttpRest::new_data_source(tx_new_data, config).await
}

#[cfg(feature = "dev")]
pub async fn new_data_source(
    tx_new_data: TxData,
    config: &str,
) -> data_source_core::Result<impl DataSourceInterface> {
    use data_source_dev::DataSourceDev;

    DataSourceDev::new_data_source(tx_new_data, config).await
}

#[cfg(feature = "special")]
pub async fn new_data_source(
    tx_new_data: TxData,
    config: &str,
) -> data_source_core::Result<impl DataSourceInterface> {
    use data_source_special::DataSourceSpecial;

    DataSourceSpecial::new_data_source(tx_new_data, config).await
}
