use serde::{Deserialize, Serialize};

// Time since unix epoch, milliseconds
pub type EpochTimeMS = u128;

#[derive(Default, Debug, Clone)]
pub struct MsgEventData {
    pub publish_time_ms: EpochTimeMS,
    pub ack_time_ms: Option<EpochTimeMS>,
    pub id: u32,
}

//#[derive(Debug, Serialize, Deserialize)]
//pub struct MsgEventList {
    //pub data: Vec<DataEvent>,
//}

#[derive(Debug, Serialize, Deserialize)]
pub enum DataEvent {
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
