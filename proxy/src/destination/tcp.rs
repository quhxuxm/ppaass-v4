use crate::error::Error;
use common::Error as CommonError;
use protocol::UnifiedAddress;
use std::io::Error as StdIoError;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio::pin;
use tokio::time::timeout;

pub struct TcpDestEndpoint {
    pub dst_addr: SocketAddr,
    tcp_stream: TcpStream,
}

impl TcpDestEndpoint {
    pub async fn connect(
        unified_dst_addr: UnifiedAddress,
        connect_timeout: u64,
    ) -> Result<Self, Error> {
        let dst_addrs: Vec<SocketAddr> = unified_dst_addr.try_into()?;
        let tcp_stream = timeout(
            Duration::from_secs(connect_timeout),
            TcpStream::connect(&dst_addrs[..]),
        )
            .await
            .map_err(|_| CommonError::ConnectTimeout(connect_timeout))??;
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
