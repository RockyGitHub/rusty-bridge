use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpecialHiveMQError {
    #[error("initialization: [{0}]")]
    Init(String),
    #[error("publish: [{0}]")]
    Publish(String),
    #[error("publish acknowledgement: [{0}]")]
    PubAck(String),
    #[error("connect: [{0}]")]
    Connect(String),
}

impl From<SpecialHiveMQError> for cloud_adapter_core::Error {
    fn from(value: SpecialHiveMQError) -> Self {
        match value {
            SpecialHiveMQError::Init(err) => cloud_adapter_core::Error::Initialization(err),
            SpecialHiveMQError::Publish(_err) => todo!(),
            SpecialHiveMQError::PubAck(_err) => todo!(),
            SpecialHiveMQError::Connect(_err) => todo!(),
        }
    }
}
