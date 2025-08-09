use common::{build_server_runtime, init_log, start_server, ServerState};
use proxy::config::get_config;
use proxy::error::Error;
use proxy::tunnel;
use tokio::signal;
use tracing::{debug, error, info};

/// Handle the incoming client connection
async fn handle_agent_connection(server_state: ServerState) -> Result<(), Error> {
    debug!("Handling agent connection: {server_state:?}.");
    tunnel::process(server_state).await?;
    Ok(())
}

/// Start the proxy server
fn main() -> Result<(), Error> {
    let _log_guard = init_log(get_config())?;
    let server_runtime = build_server_runtime(get_config())?;
    server_runtime.block_on(async move {
        let server_guard = start_server(get_config(), handle_agent_connection);
        if let Err(e) = signal::ctrl_c().await {
            error!("Error happen when listening stop signal: {}", e);
            return;
        }
        info!("Receive stop signal, going to stop server gracefully.");
        server_guard.stop_signal.cancel();
    });
    Ok(())
}
