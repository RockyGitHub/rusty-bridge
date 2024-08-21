use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct SpecialEnvelope {
    #[serde(rename = "data")]
    data: Data,
    #[serde(rename = "timestampMs")]
    timestamp_ms: String,
    #[serde(rename = "version")]
    version: String,
}

#[derive(Serialize, Deserialize)]
struct Data {
    message: String,
}

impl SpecialEnvelope {
    pub fn new(data: String) -> SpecialEnvelope {
        let data = Data { message: data };

        SpecialEnvelope {
            data: data,
            timestamp_ms: "2024-04-02T12:58:45.231140149Z".to_string(),
            version: "1.0.0".to_string(),
        }
    }
}

pub const EXAMPLE_SPECIAL_MSG: &str = r#"
{
  "data": {
  },
  "timestamp_ms": "2024-04-02T12:58:45.231140149Z",
  "version": "1.0.0"
}
"#;
