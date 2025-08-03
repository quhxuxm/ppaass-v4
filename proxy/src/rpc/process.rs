use crate::error::Error;
use crate::rpc::session::{CreateSessionDto, SessionManager};
use crate::rpc::tunnel::Tunnel;
use ppaass_common::{generate_uuid, random_bytes, Encryption};
use ppaass_rpc::rpc_process_server::RpcProcess;
use ppaass_rpc::{
    ConnectRemoteRequest, ConnectRemoteResponse, CreateSessionRequest, CreateSessionResponse,
    EncryptionType, RelayData,
};
use tokio::sync::mpsc::channel;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::codegen::tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};
use tracing::{debug, error};

pub struct RpcProcessImpl {
    session_manager: SessionManager,
}
impl RpcProcessImpl {
    pub fn new() -> Self {
        Self {
            session_manager: Default::default(),
        }
    }
}

#[tonic::async_trait]
impl RpcProcess for RpcProcessImpl {
    async fn create_session(
        &self,
        rpc_request: Request<CreateSessionRequest>,
    ) -> Result<Response<CreateSessionResponse>, Status> {
        let CreateSessionRequest {
            username,
            agent_encryption_type,
            agent_encryption,
        } = rpc_request.into_inner();
        let rpc_agent_encryption_type =
            EncryptionType::try_from(agent_encryption_type).map_err(|e| {
                Status::invalid_argument(format!("Fail to parse agent encryption type: {e}"))
            })?;
        let agent_encryption = match rpc_agent_encryption_type {
            EncryptionType::Plain => Encryption::Plain,
            EncryptionType::Aes => Encryption::Aes(agent_encryption),
            EncryptionType::Blowfish => Encryption::Blowfish(agent_encryption),
        };
        let proxy_encryption = Encryption::Aes(random_bytes::<32>().to_vec());

        let create_session_dto = CreateSessionDto {
            username,
            agent_encryption,
            proxy_encryption,
        };
        let session_id = self.session_manager.create_session(create_session_dto);
        let session =
            self.session_manager
                .fetch_session(&session_id)
                .ok_or(Error::SessionManager(format!(
                    "Can not find session just created: {session_id}"
                )))?;
        let (rpc_encryption_type, rpc_proxy_encryption) = match &session.value().proxy_encryption {
            Encryption::Aes(encryption) => (EncryptionType::Aes, encryption.clone()),
            Encryption::Blowfish(encryption) => (EncryptionType::Blowfish, encryption.clone()),
            Encryption::Plain => (EncryptionType::Plain, vec![]),
        };

        let create_session_response = CreateSessionResponse {
            proxy_encryption_type: rpc_encryption_type.into(),
            proxy_encryption: rpc_proxy_encryption,
            session_id: session.id.clone(),
        };
        Ok(Response::new(create_session_response))
    }
    async fn connect_remote(
        &self,
        rpc_request: Request<ConnectRemoteRequest>,
    ) -> Result<Response<ConnectRemoteResponse>, Status> {
        let ConnectRemoteRequest {
            session_id,
            remote_address,
        } = rpc_request.into_inner();
        let mut session =
            self.session_manager
                .fetch_session_mut(&session_id)
                .ok_or(Status::not_found(format!(
                    "Can not find session: {session_id}"
                )))?;

        let tunnel_id = generate_uuid();
        let (remote_to_agent_tx, mut remote_to_agent_rx) = channel(1024);
        let tunnel = Tunnel::new(
            remote_address.try_into().map_err(|e| {
                Status::invalid_argument(format!("Fail to parse remote address: {e}"))
            })?,
            remote_to_agent_tx,
        )
        .await?;
        let receiver_stream = ReceiverStream::new(remote_to_agent_rx);
        session
            .value_mut()
            .tunnels
            .insert(tunnel_id.clone(), tunnel);
        Ok(Response::new(ConnectRemoteResponse {
            session_id,
            tunnel_id,
        }))
    }
    type RelayStream = ReceiverStream<Result<RelayData, Status>>;
    async fn relay(
        &self,
        rpc_request: Request<Streaming<RelayData>>,
    ) -> Result<Response<Self::RelayStream>, Status> {
        let mut rpc_agent_to_remote_stream = rpc_request.into_inner();
        loop {
            let rpc_agent_to_remote_relay_data = match rpc_agent_to_remote_stream.next().await {
                None => {
                    break;
                }
                Some(Err(e)) => {
                    error!("Fail to read rpc agent to remote stream: {e}");
                    break;
                }
                Some(Ok(relay_data)) => relay_data,
            };
            let Some(session) = self
                .session_manager
                .fetch_session(&rpc_agent_to_remote_relay_data.session_id)
            else {
                error!(
                    "Can not found session: {}",
                    rpc_agent_to_remote_relay_data.session_id
                );
                continue;
            };
            let Some(tunnel) = session
                .tunnels
                .get(&rpc_agent_to_remote_relay_data.tunnel_id)
            else {
                error!(
                    "Can not found tunnel: {}",
                    rpc_agent_to_remote_relay_data.tunnel_id
                );
                continue;
            };
            if rpc_agent_to_remote_relay_data.finish {
                debug!(
                    "Transfer agent to remote finish on tunnel: {}",
                    rpc_agent_to_remote_relay_data.tunnel_id
                );
                session
                    .tunnels
                    .remove(&rpc_agent_to_remote_relay_data.tunnel_id);
                continue;
            } else {
                if let Err(e) = tunnel
                    .send_to_remote(rpc_agent_to_remote_relay_data.encrypted_payload)
                    .await
                {
                    error!(
                        "Fail to relay rpc data to remote tunnel: {}",
                        rpc_agent_to_remote_relay_data.tunnel_id
                    );
                    continue;
                };
            }
        }
        todo!()
    }
}
