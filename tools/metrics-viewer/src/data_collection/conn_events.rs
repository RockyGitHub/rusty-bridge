use serde::{Deserialize, Serialize};

use super::msg_events::EpochTimeMS;




#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnEventData {
    pub utc_time: EpochTimeMS,
    pub connected: bool,
}