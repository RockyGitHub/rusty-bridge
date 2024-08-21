use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("get config: [{0}]")]
    GetConfig(String),
}
