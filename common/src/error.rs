use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Fail to parse unified address: {0}")]
    ParseUnifiedAddress(String),
}
