use msg_transform_core::MsgTransform;

pub struct TransformDev {}

impl MsgTransform for TransformDev {
    fn transform(payload: String) {
        println!("transforming");
    }
}

impl TransformDev {
    pub fn new() -> TransformDev {
        TransformDev {}
    }
}
