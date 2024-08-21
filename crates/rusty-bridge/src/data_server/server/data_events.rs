use serde::{Deserialize, Serialize};

// Time since unix epoch, milliseconds
pub type EpochTimeMS = u128;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataEvent {
    ConnectionEvent(ConnectionEvent),
    NewMsg {
        id: u32,
        utc_time: EpochTimeMS,
    },
    PubMsg {
        id: u32,
        utc_time: EpochTimeMS,
    },
    AckMsg {
        id: u32,
        utc_time: EpochTimeMS,
        success: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEvent {
    pub utc_time: EpochTimeMS,
    pub connected: bool,
}
