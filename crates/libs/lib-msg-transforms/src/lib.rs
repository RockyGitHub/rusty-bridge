use msg_transform_core::MsgTransform;
use msg_transform_dev::TransformDev;

pub fn init_msg_transformer() -> impl MsgTransform + Send {
    // TODO - wrap possible choices in enum and choose from there
    TransformDev::new()
    //TransformSpecial::new()
}
