use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("initialize [{0}]")]
    Initialize(String),
    #[error("publish")]
    Publish,
    #[error("reserved {0}")]
    Reserved(String),
}
