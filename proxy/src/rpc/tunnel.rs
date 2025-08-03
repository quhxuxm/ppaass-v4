use crate::error::Error;
use ppaass_common::{generate_uuid, UnifiedAddress};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;
use tracing::error;

pub struct TunnelToAgentData {
    pub tunnel_id: String,
    pub data: Vec<u8>,
}
pub struct Tunnel {
    pub id: String,
    remote_output_tx: Sender<Vec<u8>>,
    remote_to_agent_task: Option<JoinHandle<()>>,
    agent_to_remote_task: Option<JoinHandle<()>>,
}

impl Tunnel {
    pub async fn new(
        remote_address: UnifiedAddress,
        agent_stream_tx: Sender<TunnelToAgentData>,
    ) -> Result<Self, Error> {
        let remote_address: Vec<SocketAddr> =
            remote_address.try_into().map_err(Error::CommonCrate)?;
        let remote_tcp_stream = TcpStream::connect(&remote_address[..]).await?;
        let (mut remote_tcp_stream_read, mut remote_tcp_stream_write) =
            remote_tcp_stream.into_split();
        let (remote_output_tx, mut remote_output_rx) = channel::<Vec<u8>>(1024);
        let tunnel_id = generate_uuid();
        let remote_to_agent_task = {
            let tunnel_id = tunnel_id.clone();
            tokio::spawn(async move {
                loop {
                    let mut remote_data_buf = [0u8; 65535];
                    match remote_tcp_stream_read.read(&mut remote_data_buf).await {
                        Ok(0) => {
                            break;
                        }
                        Ok(size) => {
                            let remote_data = &remote_data_buf[..size];
                            if let Err(_) = agent_stream_tx
                                .send(TunnelToAgentData {
                                    tunnel_id: tunnel_id.clone(),
                                    data: remote_data.to_vec(),
                                })
                                .await
                            {
                                error!("Fail to send remote data to agent in tunnel: {tunnel_id}");
                            };
                        }
                        Err(e) => {
                            error!("Fail to read remote data: {e:?}");
                            break;
                        }
                    };
                }
            })
        };
        let agent_to_remote_task = {
            let tunnel_id = tunnel_id.clone();
            tokio::spawn(async move {
                while let Some(remote_data) = remote_output_rx.recv().await {
                    if let Err(e) = remote_tcp_stream_write.write_all(&remote_data).await {
                        error!("Tunnel [{tunnel_id}] fail to write remote data: {e:?}");
                    };
                }
            })
        };
        Ok(Self {
            id: tunnel_id,
            remote_output_tx,
            remote_to_agent_task: Some(remote_to_agent_task),
            agent_to_remote_task: Some(agent_to_remote_task),
        })
    }

    pub async fn send_to_remote(&self, data: Vec<u8>) -> Result<(), Error> {
        self.remote_output_tx
            .send(data)
            .await
            .map_err(|_| Error::Tunnel(format!("Fail to send data to remote: {}", self.id)))?;
        Ok(())
    }
}

impl Drop for Tunnel {
    fn drop(&mut self) {
        let agent_to_remote_task = self.agent_to_remote_task.take().expect(&format!(
            "Fail to drop agent to remote task on tunnel: {}",
            self.id
        ));
        let remote_to_agent_task = self.remote_to_agent_task.take().expect(&format!(
            "Fail to drop remote to agent task on tunnel: {}",
            self.id
        ));
        tokio::spawn(async move {
            tokio::join!(agent_to_remote_task, remote_to_agent_task);
        });
    }
}
