mod http;
mod socks5;

use crate::config::get_config;
use crate::error::Error;
use crate::user::get_agent_user_repo;
use common::proxy::{ProxyConnection, ProxyFramed};
use common::user::UserRepository;
use common::{ServerState, WithUsernameConfig};
use tokio::io::AsyncWriteExt;
use tokio::sync::oneshot::channel;
use tracing::{debug, error};

const SOCKS4_VERSION_FLAG: u8 = 4;
const SOCKS5_VERSION_FLAG: u8 = 5;

pub async fn process(mut server_state: ServerState) -> Result<(), Error> {
    let mut protocol_flag_buf = [0u8; 1];
    let flag_size = server_state
        .incoming_stream
        .peek(&mut protocol_flag_buf)
        .await?;
    if flag_size == 0 {
        return Ok(());
    }
    let protocol_flag = protocol_flag_buf[0];
    match protocol_flag {
        SOCKS4_VERSION_FLAG => {
            error!("Socks 4 protocol not supported");
            server_state.incoming_stream.shutdown().await?;
        }
        SOCKS5_VERSION_FLAG => {
            debug!(
                "Accept socks 5 protocol client connection [{}].",
                server_state.incoming_connection_addr
            );
            socks5::process_socks5_tunnel(server_state).await?;
        }
        _ => {
            debug!(
                "Accept http/https protocol client connection [{}].",
                server_state.incoming_connection_addr
            );
            http::process_http_tunnel(server_state).await?;
        }
    }
    Ok(())
}

async fn fetch_proxy_connection<'a>() -> Result<ProxyConnection<ProxyFramed<'a>>, Error>
{
    let config = get_config();
    let agent_user = get_agent_user_repo()
        .find_user(config.username())
        .ok_or(common::Error::UserNotExist(config.username().to_owned()))?;
    let (proxy_connection_tx, proxy_connection_rx) = channel();
    tokio::spawn(async move {
        let connection = match ProxyConnection::new(agent_user, config.proxy_connect_timeout()).await.map_err(Error::Common) {
            Ok(connection) => connection,
            Err(e) => {
                error!("Fail to initialize proxy connection: {e:?}");
                return;
            }
        };
        if proxy_connection_tx.send(connection).is_err() {
            error!("Fail to send proxy connection to channel");
        }
    });
    let proxy_connection = proxy_connection_rx.await.map_err(|_| Error::Unknown("Fail to receive proxy connection from channel".to_string()))?;
    Ok(proxy_connection)
}