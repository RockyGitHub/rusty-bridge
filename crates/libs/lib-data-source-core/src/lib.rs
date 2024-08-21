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
pub mod error;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use async_trait::async_trait;
use error::Error;

#[derive(Debug, Clone)]
pub struct MsgBusData {
    pub id: u32,
    // payload should be a &[u8] but for the ease of POC, leaving it as a Vec<u8> for now
    pub payload: Vec<u8>,
    pub retry_count: u32,
}
//pub struct MsgBusData<'a> {
//pub data: &'a str,
//}

// Hopefully implement a ringbuffer with this at some point. For now it will just wrap a tokio channel
#[derive(Clone)]
pub struct TxData {
    tx: tokio::sync::mpsc::Sender<MsgBusData>,
    seq: Arc<AtomicU32>,
}

/// Receiver of data originating from data source
pub struct RxData {
    rx: tokio::sync::mpsc::Receiver<MsgBusData>,
}

// Initialized so retry_count can be defined by an env variable. Maybe other things too? Like
pub struct MsgBusDataFactory {
    retry_count: u32,
}

#[derive(Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub custom: Option<String>,
}

pub struct ConfigData {
    /// The cloud adapters to use (only supporting 1 right now)
    pub adapters: Vec<String>,
    /// Message topics to subscribe to
    pub subscription_topics: Vec<String>,
    /// Unsure the purpose here yet
    pub publish_topics: Vec<String>,
    pub publication_parameters: String, // Some custom json type?
    pub credentials: Credentials,
}

pub type Result<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait DataSourceInterface {
    // TODO - how can config type change to a concrete type instead of being an issue at run time if wrong?
    async fn new_data_source(tx_new_data: TxData, config: &str)
        -> Result<impl DataSourceInterface>;
}

impl TxData {
    pub fn new() -> (TxData, RxData) {
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        let tx = TxData {
            tx,
            seq: Arc::new(AtomicU32::new(0)),
        };
        let rx = RxData { rx };
        (tx, rx)
    }

    /// TODO - should take in &[u8], copy it to a ring buffer I think, and then recv will convert it to MsgBusData?
    pub fn send(&self, data: &[u8]) {
        // TODO
        // Copy to a ring buffer location
        let payload = data.to_vec(); // do this instead for now
                                     // Pass the pointer to the channel
        let id = self.seq.fetch_add(1, Ordering::Relaxed);
        let data = MsgBusData {
            id,
            payload,
            retry_count: 0,
        };

        // TODO - if this occurs there's a major issue.  This should either trigger a self-heal event or pass to a backup channel/storage
        let _res = self
            .tx
            .try_send(data)
            //.await
            .map_err(|err| Error::Reserved(err.to_string()));
    }
}

impl RxData {
    pub async fn recv(&mut self) -> Option<MsgBusData> {
        // todo
        // Get the latest pointer to the ring buffer
        self.rx.recv().await
    }
}

impl MsgBusDataFactory {
    pub fn new() -> MsgBusDataFactory {
        // TODO - fetch env varaible
        MsgBusDataFactory { retry_count: 1 }
    }

    pub fn msg(&self, data: &[u8]) -> MsgBusData {
        MsgBusData {
            id: 0,
            payload: data.into(),
            retry_count: self.retry_count,
        }
    }
}
