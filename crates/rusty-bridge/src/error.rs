use thiserror::Error;

use crate::shutdown::termination::ExitReason;

pub type Result<T> = std::result::Result<T, RustyBridgeError>;

#[derive(Error, Debug)]
pub enum RustyBridgeError {
    #[error("initialization failed because [{0}]")]
    Initialization(String),
    #[error("edge reporter: [{0}]")]
    EdgeReporter(String),
    #[error("reserved")]
    Reserved,
}

impl From<RustyBridgeError> for ExitReason {
    fn from(value: RustyBridgeError) -> Self {
        match value {
            RustyBridgeError::Initialization(_) => Self::Failure,
            _ => Self::Unknown,
        }
    }
}
