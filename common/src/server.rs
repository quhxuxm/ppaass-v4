use crate::config::ServerConfig;
use crate::error::Error;
use std::error::Error as StdError;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

#[derive(Debug)]
pub struct ServerState {
    pub incoming_stream: TcpStream,
    pub incoming_connection_addr: SocketAddr,
}

pub struct ServerGuard {
    pub stop_signal: CancellationToken,
}

pub fn start_server<C, F, Fut, Err>(config: &C, connection_handler: F) -> ServerGuard
where
    C: ServerConfig,
    F: Fn(ServerState) -> Fut + Send + Sync + Copy + 'static,
    Fut: Future<Output = Result<(), Err>> + Send + 'static,
    Err: StdError + From<Error>,
{
    let stop_single = CancellationToken::new();
    let server_guard = ServerGuard {
        stop_signal: stop_single.clone(),
    };
    let listening_address = config.listening_address();
    let client_max_connections = Arc::new(Semaphore::new(config.client_max_connections()));
    tokio::spawn(async move {
        let tcp_listener = match TcpListener::bind(listening_address).await {
            Ok(tcp_listener) => tcp_listener,
            Err(e) => {
                error!("Fail to bind server [{listening_address}] because of error: {e:?}");
                return;
            }
        };
        loop {
            tokio::select! {
                _ = stop_single.cancelled() => {
                    info!("Receive stop signal, stop server success.");
                    return;
                }
                client_connection = tcp_listener.accept() => {
                    let client_connection_permit=match client_max_connections.clone().acquire_owned().await{
                        Ok(client_connection_permit) => client_connection_permit,
                        Err(e) => {
                            error!("Fail to acquire client connection permit because of error: {e:?}");
                            continue;
                        }
                    };
                    let (incoming_stream, incoming_connection_addr) = match client_connection {
                        Ok((incoming_stream, incoming_connection_addr)) => (incoming_stream, incoming_connection_addr),
                        Err(e) => {
                            error!("Failed to accept incoming connection: {}", e);
                            continue;
                        }
                    };
                    debug!("Accept incoming connection from {}", incoming_connection_addr);
                    tokio::spawn(async move {
                        let server_state = ServerState {
                            incoming_stream,
                            incoming_connection_addr,
                        };
                        if let Err(e) = connection_handler(server_state).await {
                            error!("Failed to handle incoming connection: {:?}", e);
                        }
                        drop(client_connection_permit);
                    });
                }
            }
        }
    });
    server_guard
}
