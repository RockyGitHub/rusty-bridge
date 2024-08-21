use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErrorSpecialAnA {
    #[error("initilization failed due to: [{0}]")]
    Initialization(String),
    #[error("fetching token failed due to: [{0}]")]
    FetchingToken(String),
    #[error("reserved")]
    Reserved,
}

impl From<jsonwebtoken::errors::Error> for ErrorSpecialAnA {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        Self::FetchingToken(value.to_string())
    }
}
