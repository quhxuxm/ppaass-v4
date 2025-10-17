use crate::user::UserWithProxyServers;
use crate::{
    Error, SecureLengthDelimitedCodec, get_handshake_encryption, random_generate_encryption,
    rsa_decrypt_encryption, rsa_encrypt_encryption,
};
use futures_util::{SinkExt, StreamExt};
use ppaass_protocol::{
    ConnectDestinationRequest, ConnectDestinationResponse, HandshakeRequest, HandshakeResponse,
    UnifiedAddress,
};
use std::borrow::Cow;
use std::io::Error as StdIoError;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio::pin;
use tokio::time::timeout;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::Framed;
use tokio_util::io::{SinkWriter, StreamReader};

pub type ProxyFramed<'a> = Framed<TcpStream, SecureLengthDelimitedCodec<'a>>;
pub type ProxyFramedReadWrite<'a> = SinkWriter<StreamReader<ProxyFramed<'a>, BytesMut>>;

pub enum DestinationType {
    Tcp,
    #[allow(unused)]
    Udp,
}

pub struct Init;

#[derive(Debug)]
/// The proxy connection.
pub struct ProxyConnection<T> {
    state: T,
}

impl ProxyConnection<Init> {
    pub async fn new<'a, U>(
        user_info: &U,
        connect_timeout: u64,
    ) -> Result<ProxyConnection<ProxyFramed<'a>>, Error>
    where
        U: UserWithProxyServers + Send + Sync + 'static,
    {
        let mut proxy_stream = timeout(
            Duration::from_secs(connect_timeout),
            TcpStream::connect(user_info.proxy_servers()),
        )
        .await
        .map_err(|_| Error::ConnectTimeout(connect_timeout))??;
        let mut handshake_framed = Framed::new(
            &mut proxy_stream,
            SecureLengthDelimitedCodec::new(
                Cow::Borrowed(get_handshake_encryption()),
                Cow::Borrowed(get_handshake_encryption()),
            ),
        );
        let agent_encryption = random_generate_encryption();
        let rsa_encrypted_agent_encryption = rsa_encrypt_encryption(
            &agent_encryption,
            user_info
                .rsa_crypto()
                .ok_or(Error::UserRsaCryptoNotExist(user_info.username().clone()))?,
        )?;
        let client_handshake_request = HandshakeRequest {
            username: user_info.username().to_owned(),
            encryption: rsa_encrypted_agent_encryption.into_owned(),
        };
        let client_handshake_request_bytes: Vec<u8> = client_handshake_request.try_into()?;
        handshake_framed
            .send(&client_handshake_request_bytes)
            .await?;
        let proxy_handshake_bytes =
            handshake_framed
                .next()
                .await
                .ok_or(Error::ConnectionExhausted(format!(
                    "Fail to read handshake message from proxy: {}",
                    proxy_stream.peer_addr()?
                )))??;
        let rsa_encrypted_proxy_handshake: HandshakeResponse = proxy_handshake_bytes.try_into()?;
        let proxy_encryption = rsa_decrypt_encryption(
            rsa_encrypted_proxy_handshake.encryption,
            user_info.rsa_crypto().ok_or(Error::UserRsaCryptoNotExist(
                user_info.username().to_owned(),
            ))?,
        )?;
        let proxy_framed = Framed::new(
            proxy_stream,
            SecureLengthDelimitedCodec::new(
                Cow::Owned(proxy_encryption),
                Cow::Owned(agent_encryption),
            ),
        );
        Ok(ProxyConnection {
            state: proxy_framed,
        })
    }
}

impl<'a> ProxyConnection<ProxyFramed<'a>> {
    pub async fn connect_destination(
        self,
        destination_addr: UnifiedAddress,
        destination_type: DestinationType,
    ) -> Result<ProxyConnection<ProxyFramedReadWrite<'a>>, Error> {
        let mut proxy_framed = self.state;
        let connect_destination_request = match destination_type {
            DestinationType::Tcp => ConnectDestinationRequest::Tcp(destination_addr.clone()),
            DestinationType::Udp => ConnectDestinationRequest::Udp(destination_addr.clone()),
        };
        let connect_destination_request_bytes: Vec<u8> = connect_destination_request.try_into()?;
        proxy_framed
            .send(&connect_destination_request_bytes)
            .await?;
        let connect_destination_response_bytes = proxy_framed
            .next()
            .await
            .ok_or(Error::ConnectionExhausted(format!("Fail to read setup destination connection message from proxy, destination address: {destination_addr:?}")))??;
        let connect_destination_response = connect_destination_response_bytes.try_into()?;
        match connect_destination_response {
            ConnectDestinationResponse::Success => Ok(ProxyConnection {
                state: SinkWriter::new(StreamReader::new(proxy_framed)),
            }),
            ConnectDestinationResponse::Fail => Err(Error::ConnectDestination(destination_addr)),
        }
    }
}

impl<'a> AsyncRead for ProxyConnection<ProxyFramedReadWrite<'a>> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let proxy_framed = &mut self.get_mut().state;
        pin!(proxy_framed);
        proxy_framed.poll_read(cx, buf)
    }
}

impl<'a> AsyncWrite for ProxyConnection<ProxyFramedReadWrite<'a>> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, StdIoError>> {
        let proxy_framed = &mut self.get_mut().state;
        pin!(proxy_framed);
        proxy_framed.poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), StdIoError>> {
        let proxy_framed = &mut self.get_mut().state;
        pin!(proxy_framed);
        proxy_framed.poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), StdIoError>> {
        let proxy_framed = &mut self.get_mut().state;
        pin!(proxy_framed);
        proxy_framed.poll_shutdown(cx)
    }
}
