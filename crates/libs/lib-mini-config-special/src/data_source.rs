use std::{collections::HashSet, env};

use mini_config_core::Error;
use tracing::warn;

use crate::SERVER_TYPE;

/// Collects the configuration required for the data_source
pub fn get_data_source(tmp: String) -> mini_config_core::Result<data_source_special::Config> {
    let topics = get_topics(tmp)?;

    // V3 sets these via env vars
    let pub_endpoint =
        env::var("ZMQ_PUB_ENDPOINT").map_err(|err| Error::GetConfig(err.to_string()))?;

    let sub_endpoint =
        env::var("ZMQ_SUB_ENDPOINT").map_err(|err| Error::GetConfig(err.to_string()))?;

    let highwater_mark = 1000; // some arbitrary

    let config = data_source_special::Config {
        pub_endpoint,
        sub_endpoint,
        highwater_mark,
        topics,
    };
    Ok(config)
}

fn get_topics(tmp: String) -> Result<Vec<String>, Error> {
    // Get topics by some proprietary way

    let topics = vec![
        "test1".to_string(),
        "test2".to_string(),
        "test3".to_string(),
    ];

    Ok(topics)
}
