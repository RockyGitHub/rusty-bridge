#[derive(Clone)]
pub struct MsgPersistence {
    pub tmp: String,
}

impl MsgPersistence {
    pub fn new() -> MsgPersistence {
        MsgPersistence {
            tmp: "hi".to_string(),
        }
    }
}
