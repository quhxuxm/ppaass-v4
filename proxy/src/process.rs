use ppaass_rpc::rpc_process_server::RpcProcess;
use ppaass_rpc::{CreateConnectionRequest, CreateConnectionResponse, CreateSessionRequest, CreateSessionResponse, RelayData};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

pub struct RpcProcessImpl;

impl RpcProcessImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[tonic::async_trait]
impl RpcProcess for RpcProcessImpl {
    async fn create_session(&self, request: Request<CreateSessionRequest>) -> Result<Response<CreateSessionResponse>, Status> {
        todo!()
    }
    async fn create_connection(&self, request: Request<CreateConnectionRequest>) -> Result<Response<CreateConnectionResponse>, Status> {
        todo!()
    }
    type relayStream = ReceiverStream<Result<RelayData, Status>>;
    async fn relay(&self, request: Request<Streaming<RelayData>>) -> Result<Response<Self::relayStream>, Status> {
        todo!()
    }
}
