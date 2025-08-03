use thiserror::Error;
use tonic::Status;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Session manager error: {0}")]
    SessionManager(String),
    #[error("Tunnel error: {0}")]
    Tunnel(String),
    #[error(transparent)]
    CommonCrate(#[from] ppaass_common::error::Error),
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        match value {
            Error::Io(e) => Status::internal(e.to_string()),
            Error::SessionManager(e) => Status::internal(e),
            Error::CommonCrate(e) => Status::internal(e.to_string()),
        }
    }
}
