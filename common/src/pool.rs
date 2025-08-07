use crate::proxy::{ProxyConnection, ProxyFramed};
use crate::user::UserWithProxyServers;
use crate::WithConnectionPoolConfig;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::{Mutex, Notify, RwLock};
use tokio::time::sleep;
use tracing::error;

pub static PROXY_CONNECTION_POOL: OnceLock<RwLock<ProxyConnectionPool>> = OnceLock::new();

pub struct ProxyConnectionPool<'a>
where
    'a: 'static,
{
    connections: Arc<Mutex<Vec<ProxyConnection<ProxyFramed<'a>>>>>,
    connections_available_notify: Arc<Notify>,
}

impl<'a> ProxyConnectionPool<'a>
where
    'a: 'static,
{
    pub fn new<U, C>(user_info: Arc<U>, config: &C) -> ProxyConnectionPool<'a>
    where
        U: UserWithProxyServers + Send + Sync + 'static,
        C: WithConnectionPoolConfig + Send + Sync + 'static,
    {
        let proxy_connect_timeout = config.proxy_connect_timeout();
        let connection_pool_size = config.connection_pool_size();
        let connections = Arc::new(Mutex::new(vec![]));
        let connections_available_notify = Arc::new(Notify::new());
        {
            let connections = connections.clone();
            let connections_available_notify = connections_available_notify.clone();
            tokio::spawn(async move {
                loop {
                    let mut connections = connections.lock().await;
                    if connections.len() >= connection_pool_size {
                        sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                    let proxy_connection = match ProxyConnection::new(user_info.clone(), proxy_connect_timeout).await {
                        Ok(proxy_connection) => proxy_connection,
                        Err(e) => {
                            error!("Fail to create proxy connection: {e}");
                            continue;
                        }
                    };
                    connections.push(proxy_connection);
                    connections_available_notify.notify_waiters();
                }
            });
        }
        ProxyConnectionPool {
            connections,
            connections_available_notify,
        }
    }
    pub async fn fetch_connection(&mut self) -> ProxyConnection<ProxyFramed<'a>> {
        loop {
            self.connections_available_notify.notified().await;
            let connection = self.connections.lock().await.pop();
            match connection {
                None => {
                    continue;
                }
                Some(connection) => return connection,
            }
        }
    }
}
