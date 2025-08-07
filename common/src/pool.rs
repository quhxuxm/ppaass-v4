use crate::proxy::{ProxyConnection, ProxyFramed};
use crate::user::UserWithProxyServers;
use crate::WithConnectionPoolConfig;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::sleep;
use tracing::error;

pub static PROXY_CONNECTION_POOL: OnceLock<ProxyConnectionPool> = OnceLock::new();

pub struct ProxyConnectionPool<'a>
where
    'a: 'static,
{
    connections: Arc<Mutex<Vec<ProxyConnection<ProxyFramed<'a>>>>>,
    connections_semaphore: Arc<Semaphore>,
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
        let connections_semaphore = Arc::new(Semaphore::new(0));
        {
            let connections = connections.clone();
            let connections_semaphore = connections_semaphore.clone();
            tokio::spawn(async move {
                loop {
                    if connections_semaphore.available_permits() >= connection_pool_size {
                        sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                    let proxy_connection = match ProxyConnection::new(user_info.clone(), proxy_connect_timeout).await {
                        Ok(proxy_connection) => proxy_connection,
                        Err(e) => {
                            error!("Fail to create proxy connection: {e}");
                            return;
                        }
                    };
                    connections.lock().await.push(proxy_connection);
                    connections_semaphore.add_permits(1);
                }
            });
        }
        ProxyConnectionPool {
            connections,
            connections_semaphore,
        }
    }
    pub async fn fetch_connection(&self) -> ProxyConnection<ProxyFramed<'a>> {
        loop {
            if let Err(e) = self.connections_semaphore.acquire().await {
                error!("Fail to acquire proxy connection pool semaphore: {e:?}");
                continue;
            };
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
