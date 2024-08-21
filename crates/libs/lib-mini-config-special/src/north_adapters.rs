use std::collections::HashSet;

use mini_config_core::Error;
use serde::{Deserialize, Serialize};

use crate::SERVER_TYPE;

#[derive(Serialize, Deserialize)]
#[serde(untagged)] // this will prevent HiveMQ or IoTHub from leading the json
pub enum NorthAdapter {
    HiveMQ(special_hivemq::Config),
    IoTHub(special_iothub::Config),
}

pub fn get_north_adapters(tmp: String) -> mini_config_core::Result<NorthAdapter> {
    // The way this data is obtained is different depending if factory provisioning or a regular protocol is used
    let servers = vec!["server1".to_string(), "server2".to_string()];

    // Multiple servers are not yet supported unfortunately
    if servers.len() > 1 {
        return Err(Error::GetConfig(format!(
            "Only 1 server is currently supported, multiple were found. [{:?}]",
            servers
        )));
    } else if servers.len() == 0 {
        return Err(Error::GetConfig(format!(
            "0 servers found, configuration is not possible"
        )));
    }
    let config = NorthAdapter::HiveMQ(special_hivemq::Config {
        username: "username".to_string(),
        password: "password".to_string(),
        ana_endpoint: "ana_endpoint".to_string(),
        mqtt_endpoint: "mqtt_endpoint".to_string(),
    });

    Ok(config)
}

type Username = String;
type Password = String;
type AnAEndpoint = String;
type MQTTEndpoint = String;

fn parse_connection_string(
    conn_string: &str,
) -> mini_config_core::Result<(Username, Password, AnAEndpoint, MQTTEndpoint)> {
    //regex is probably more appropriate here, but at least this prevents any accidental situations of the regex not workig new unknown character combinations
    let mut split = conn_string.split(";");

    let mut mqtt = split
        .next()
        .ok_or(Error::GetConfig(format!(
            "missing value ?hostname? in connection string. [{}]",
            conn_string
        )))?
        .replace("hostname=", "")
        .replace(":443", "");
    mqtt.push_str("/mqtt");

    let username = split
        .next()
        .ok_or(Error::GetConfig(format!(
            "missing value ?username? in connection string. [{}]",
            conn_string
        )))?
        .replace("username=", "");
    let password = split
        .next()
        .ok_or(Error::GetConfig(format!(
            "missing value ?password? from connection string. [{}]",
            conn_string
        )))?
        .replace("password=", "");
    let mut ana = split
        .next()
        .ok_or(Error::GetConfig(format!(
            "missing value ?ana? from connection string. [{}]",
            conn_string
        )))?
        .replace("ana=", "");
    ana.push_str("/api/v1/hivemq");

    Ok((username, password, ana, mqtt))
}
