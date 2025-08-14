use ppaass_crypto::Error as CryptoError;
use ppaass_protocol::{UnifiedAddress, Username};
use thiserror::Error;
use tracing::metadata::ParseLevelError;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    ParseLevel(#[from] ParseLevelError),
    #[error(transparent)]
    Crypto(#[from] CryptoError),
    #[error("User not exist: {0:?}")]
    UserNotExist(Username),
    #[error("User rsa crypto not exist: {0:?}")]
    UserRsaCryptoNotExist(Username),
    #[error("Connection exhausted: [{0}]")]
    ConnectionExhausted(String),
    #[error("Fail to setup destination: [{0}]")]
    ConnectDestination(UnifiedAddress),
    #[error("Connect to remote endpoint timeout in {0} seconds.")]
    ConnectTimeout(u64),
    #[error("Lock error: [{0}]")]
    Lock(String),
    #[error(transparent)]
    Protocol(#[from] ppaass_protocol::Error),
}

impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        std::io::Error::other(format!("{value:?}"))
    }
}
