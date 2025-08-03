mod error;
mod rpc;

use crate::error::Error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio::net::TcpListener;
use tokio::runtime::Builder;
use tracing::{debug, error};

async fn start_server() -> Result<(), Error> {
    let tcp_listener = TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::UNSPECIFIED,
        10081,
    )))
    .await?;
    loop {
        let (agent_tcp_stream, agent_socket_addr) = tcp_listener.accept().await?;
        debug!("Agent tcp stream connected: {agent_socket_addr}");
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .worker_threads(32)
        .build()?;
    runtime.block_on(async move {
        if let Err(e) = start_server().await {
            error!("Fail to start server: {e:?}");
        }
    });
    Ok(())
}
