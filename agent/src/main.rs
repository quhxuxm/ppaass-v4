mod command;
mod config;
mod error;
mod tunnel;
mod user;

use crate::config::get_config;
use crate::error::Error;
use crate::user::get_agent_user_repo;
use common::pool::{ProxyConnectionPool, PROXY_CONNECTION_POOL};
use common::user::UserRepository;
use common::{build_server_runtime, init_log, start_server, ServerState, WithUsernameConfig};
use tokio::signal;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

async fn handle_connection(server_state: ServerState) -> Result<(), Error> {
    debug!("Handling agent connection: {server_state:?}.");
    tunnel::process(server_state).await?;
    Ok(())
}

fn main() -> Result<(), Error> {
    let _log_guard = init_log(get_config())?;
    let server_runtime = build_server_runtime(get_config())?;
    server_runtime.block_on(async move {
        let config = get_config();
        let user_info = get_agent_user_repo()
            .find_user(config.username())
            .expect("Can not find user");
        let proxy_connection_pool = ProxyConnectionPool::new(user_info, config);
        if let Err(_) = PROXY_CONNECTION_POOL.set(RwLock::new(proxy_connection_pool)) {
            error!("Fail to store proxy connection pool into once lock");
            return;
        };
        let server_guard = start_server(get_config(), handle_connection);
        if let Err(e) = signal::ctrl_c().await {
            error!("Error happen when listening stop signal: {}", e);
            return;
        }
        info!("Receive stop signal, going to stop server.");
        server_guard.stop_signal.cancel();
    });
    Ok(())
}
