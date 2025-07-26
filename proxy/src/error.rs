use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    TonicTransport(#[from] tonic::transport::Error),
    #[error("Destination connection pool error.")]
    DestinationConnectionPool,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
