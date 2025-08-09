use crate::error::Error;
use common::Error as CommonError;
use futures_util::SinkExt;
use protocol::UnifiedAddress;
use std::io::Error as StdIoError;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio::pin;
use tokio::sync::oneshot::channel;
use tokio::time::timeout;
use tracing::error;

pub struct TcpDestEndpoint {
    pub dst_addr: SocketAddr,
    tcp_stream: TcpStream,
}

impl TcpDestEndpoint {
    pub async fn connect(
        unified_dst_addr: UnifiedAddress,
        connect_timeout: u64,
    ) -> Result<Self, Error> {
        let (dst_connection_tx, dst_connection_rx) = channel();
        tokio::spawn(async move {
            let dst_addrs: Vec<SocketAddr> = match unified_dst_addr.try_into() {
                Ok(dst_addrs) => dst_addrs,
                Err(e) => {
                    error!("Fail to convert destination address: {e:?}");
                    return;
                }
            };
            let tcp_stream = match timeout(
                Duration::from_secs(connect_timeout),
                TcpStream::connect(&dst_addrs[..]),
            )
                .await
                .map_err(|_| CommonError::ConnectTimeout(connect_timeout)) {
                Ok(Ok(tcp_stream)) => tcp_stream,
                Ok(Err(e)) => {
                    error!("Fail to connect destination {dst_addrs:?} because of error: {e}");
                    return;
                }
                Err(_) => {
                    error!("Connect to destination timeout.");
                    return;
                }
            };
            if dst_connection_tx.send(tcp_stream).is_err() {
                error!("Fail to send destination tcp stream");
            };
        });
        let tcp_stream = dst_connection_rx.await.map_err(|e| Error::Unknown("Fail to receive destination tcp stream".to_string()))?;
        let dst_addr = tcp_stream.peer_addr()?;
        Ok(Self {
            dst_addr,
            tcp_stream,
        })
    }
}

impl AsyncRead for TcpDestEndpoint {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let tcp_stream = &mut self.get_mut().tcp_stream;
        pin!(tcp_stream);
        tcp_stream.poll_read(cx, buf)
    }
}

impl AsyncWrite for TcpDestEndpoint {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, StdIoError>> {
        let tcp_stream = &mut self.get_mut().tcp_stream;
        pin!(tcp_stream);
        tcp_stream.poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), StdIoError>> {
        let tcp_stream = &mut self.get_mut().tcp_stream;
        pin!(tcp_stream);
        tcp_stream.poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), StdIoError>> {
        let tcp_stream = &mut self.get_mut().tcp_stream;
        pin!(tcp_stream);
        tcp_stream.poll_shutdown(cx)
    }
}
