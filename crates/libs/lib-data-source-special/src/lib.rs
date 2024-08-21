use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use data_source_core::{error::Error, DataSourceInterface, Result, TxData};
use serde::{Deserialize, Serialize};
use tracing::debug;

pub struct DataSourceSpecial {
    tmp: String,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub pub_endpoint: String,
    pub sub_endpoint: String,
    pub highwater_mark: u32,
    pub topics: Vec<String>,
}

#[async_trait]
impl DataSourceInterface for DataSourceSpecial {
    async fn new_data_source(tx_new_data: TxData, config: &str) -> Result<DataSourceSpecial> {
        // TODO - how can this be a concrete type instead of passed by str?
        let config = serde_json::from_str::<Config>(config)
            .map_err(|err| Error::Initialize(err.to_string()))?;

        let Config {
            pub_endpoint,
            sub_endpoint,
            highwater_mark,
            topics,
        } = config;

        let highwater_mark: i32 = highwater_mark.try_into().map_err(|err| {
            Error::Initialize(format!(
                "can't convert highwater_mark u32 to i32. [{}]",
                err
            ))
        })?;
        //let databus_client = DatabusClient::new(&pub_endpoint, &sub_endpoint, highwater_mark)
        //.map_err(|err| Error::Initialize(err.to_string()))?;

        for topic in topics {
            let callback = Arc::new(Mutex::new(TopicCallback::new(tx_new_data.clone())));
            //let topic = Topic::new().with_name(topic);

            //match databus_client.subscribe_to_topic(&topic, callback) {
            //Ok(topic) => info!("Subscribed to topic: [{}]", topic.topic_name),
            //Err(err) => error!("Could not subscribe to topic: [{}]", err),
            //}
        }

        Ok(DataSourceSpecial {
            tmp: "tmp".to_string(),
        })
    }
}

struct TopicCallback {
    tx_new_data: TxData,
}

//impl TopicSubscriberCallback for TopicCallback {
//fn on_topic_data(&self, topic_data: TopicData) {
//debug!(
//"New intake msg: [topic: {}, timestamp: {}, metadata: {:?}]",
//topic_data.topic_name, topic_data.timestamp, topic_data.metadata
//);
//trace!("Msg rawdata: [{:?}]", topic_data.rawdata);
//self.tx_new_data.send(topic_data.rawdata)
//}
//}

impl TopicCallback {
    fn new(tx_new_data: TxData) -> TopicCallback {
        TopicCallback { tx_new_data }
    }
}

impl Drop for DataSourceSpecial {
    fn drop(&mut self) {
        debug!("Dropping Databus Client");
        //match self.databus_client.close() {
        //Ok(_) => debug!("Databus client closed"),
        //Err(err) => warn!("Issue dropping databus client. [{}]", err),
        //}
    }
}
