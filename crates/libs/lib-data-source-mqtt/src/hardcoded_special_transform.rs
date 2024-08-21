use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct SpecialEnvelope {
    data: Data,
}

#[derive(Serialize, Deserialize)]
struct Data {
    message: String,
}

impl SpecialEnvelope {
    pub fn new(data: String) -> SpecialEnvelope {
        let data = Data { message: data };

        SpecialEnvelope { data: data }
    }
}

pub const EXAMPLE_SPECIAL_MSG: &str = r#"
{
  "data": {
  }
}
"#;
