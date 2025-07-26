mod error;
mod process;
mod config;

use crate::error::Error;
use crate::process::RpcProcessImpl;
use ppaass_rpc::rpc_process_server::RpcProcessServer;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::runtime::Builder;
use tonic::{codec::CompressionEncoding, transport::Server};
use tracing::error;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = Builder::new_multi_thread()
        .worker_threads(32)
        .enable_all()
        .build()?;
    runtime.block_on(async move {
        if let Err(e) = rpc_server().await {
            error!("Fail to serve rpc process because of error: {e:?}");
        }
    });
    Ok(())
}

async fn rpc_server() -> Result<(), Error> {
    let rpc_process_impl = RpcProcessImpl::new();
    let rpc_process_server = RpcProcessServer::new(rpc_process_impl)
        .accept_compressed(CompressionEncoding::Gzip)
        .send_compressed(CompressionEncoding::Gzip);
    Server::builder()
        .concurrency_limit_per_connection(32)
        .add_service(rpc_process_server)
        .serve(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8777))
        .await?;
    Ok(())
}
