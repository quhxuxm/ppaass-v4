//!
//! This module provides a `ProxyConnection` struct for establishing and managing
//! secure connections to proxy servers. The connection process involves a handshake
//! with the proxy server, followed by setting up a connection to the final destination.
//!
//! # Overview
//!
//! - **Handshake**: A secure handshake is performed with the proxy server using RSA encryption
//!   to exchange symmetric encryption keys that will be used for the rest of the communication.
//! - **Connecting to Destination**: After the handshake, the client can request the proxy
//!   to connect to a specified destination. The proxy then establishes the connection and
//!   relays data between the client and the destination.
//!
//! # Usage
//!
//! 1. Create a new `ProxyConnection` instance using the `new` method, which performs the
//!    initial handshake with the proxy server.
//! 2. Use the `connect_destination` method to request the proxy to connect to a specific
//!    destination.
//! 3. Once connected, the `ProxyConnection` can be used as an `AsyncRead` and `AsyncWrite`
//!    to read from and write to the destination through the proxy.
//!
//! # Example
//!
//! ```rust
//! use ppaass_protocol::UnifiedAddress;
//! use your_module::ProxyConnection; // Replace with the actual module path
//! use your_module::DestinationType; // Replace with the actual module path
//! use your_module::UserWithProxyServers; // Replace with the actual trait path
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let user_info = /* Your user info implementation */;
//!     let connect_timeout = 5; // Timeout in seconds
//!
//!     let mut connection = ProxyConnection::new(&user_info, connect_timeout).await?;
//!     let destination_addr = UnifiedAddress::Tcp("example.com:80".parse()?);
//!     connection.connect_destination(destination_addr, DestinationType::Tcp).await?;
//!
//!     // Now you can read and write to the destination through the proxy
//!     Ok(())
//! }
//! ```
//!
//! # Errors
//!
//! - `Error::ConnectTimeout`: If the connection to the proxy server times out.
//! - `Error::ConnectionExhausted`: If the connection to the proxy or destination is unexpectedly closed.
//! - `Error::UserRsaCryptoNotExist`: If the user's RSA crypto information is not available.
//! - `Error::ConnectDestination`: If the proxy fails to connect to the specified destination.
//!
//! # Type Definitions
//!
//! - `ProxyFramed<'a>`: A framed TCP stream with a `SecureLengthDelimitedCodec`.
//! - `ProxyFramedReaderWriter<'a>`: A combination of a reader and writer for the framed stream.
//! - `DestinationType`: An enum representing the type of destination (TCP or UDP).
//!
//! # Structs
//!
//! - `Init`: Initial state for the `ProxyConnection`.
//! - `ProxyConnection<T>`: The main struct for managing the proxy connection.
//!
//! # Enums
//!
//! - `DestinationType`: Enumerates the types of destinations (TCP, UDP).
//!
//! # Implementations
//!
//! - `ProxyConnection<Init>`: Methods for initializing the connection.
//! - `ProxyConnection<ProxyFramed<'a>>`: Methods for connecting to a destination.
//! - `ProxyConnection<ProxyFramedReaderWriter<'a>>`: Implements `AsyncRead` and `AsyncWrite` for reading and writing to the destination.
//!
//! # Notes
//!
//! - Ensure that the `UserWithProxyServers` trait is implemented for the user information
//!   structure to provide necessary details like proxy servers and RSA crypto.
//! - The `SecureLengthDelimitedCodec` is used to encode and decode messages with a secure length prefix.
//! - The `ProxyConnection` struct transitions through different states, starting from `Init`
//!   to `ProxyFramed` and finally to `ProxyFramedReaderWriter` after successfully connecting to a destination.
//!
use crate::user::UserWithProxyServers;
use crate::{
    get_handshake_encryption, random_generate_encryption, rsa_decrypt_encryption, rsa_encrypt_encryption,
    Error, SecureLengthDelimitedCodec,
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

/// Represents the type of destination for network communication.
///
/// # Variants
///
/// * `Tcp` - Indicates that the destination is a TCP (Transmission Control Protocol) endpoint.
/// * `Udp` - Indicates that the destination is a UDP (User Datagram Protocol) endpoint. This variant is currently marked as unused, suggesting it may not be fully implemented or utilized in the current context.
///
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
    ///
    /// Asynchronously creates a new `ProxyConnection` with the specified user information and connection timeout.
    ///
    /// # Arguments
    /// * `user_info` - A reference to an object implementing `UserWithProxyServers`, which contains the necessary
    ///   information for establishing the connection, including proxy servers, RSA crypto, and username.
    /// * `connect_timeout` - The timeout in seconds for the initial TCP connection attempt.
    ///
    /// # Returns
    /// A `Result` containing either a `ProxyConnection` that encapsulates the established secure communication channel,
    /// or an `Error` if the connection fails or any other error occurs.
    ///
    /// # Errors
    /// This function can return an `Error` variant for various failure conditions, such as:
    /// - `ConnectTimeout` when the connection attempt exceeds the specified timeout.
    /// - `UserRsaCryptoNotExist` if the user's RSA crypto information is missing.
    /// - `ConnectionExhausted` if the handshake process fails due to an unexpected end of the stream.
    /// - Other errors related to encryption, decryption, or message serialization/deserialization.
    ///
    /// # Panics
    /// This function does not panic under normal circumstances. However, panics may occur if there are bugs in
    /// the underlying cryptographic or network libraries.
    ///
    /// # Examples
    /// ```no_run
    /// use your_module::new; // Replace with the actual path to this function
    /// use your_module::UserWithProxyServers; // Example trait, replace with the actual trait
    /// use your_module::Error; // Example error type, replace with the actual error type
    ///
    /// # async fn example() -> Result<(), Error> {
    /// let user_info = ...; // Initialize with a valid UserWithProxyServers implementation
    /// let connect_timeout = 10; // Timeout in seconds
    /// let connection = new(&user_info, connect_timeout).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Notes
    /// - The function establishes a TCP connection to one of the proxy servers provided by `user_info`.
    /// - It then performs a secure handshake using RSA encryption to exchange symmetric keys.
    /// - After the handshake, it sets up a framed transport layer for secure communication.
    /// - The `ProxyConnection` returned can be used to send and receive messages over the secure channel.
    ///
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
            user_info.rsa_crypto().ok_or(Error::UserRsaCryptoNotExist(
                user_info.username().clone(),
            ))?,
        )?;
        let client_handshake_request = HandshakeRequest {
            username: user_info.username().to_owned(),
            encryption: rsa_encrypted_agent_encryption.into_owned(),
        };
        let client_handshake_request_bytes: Vec<u8> = client_handshake_request.try_into()?;
        handshake_framed.send(&client_handshake_request_bytes).await?;
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
    /// Asynchronously establishes a connection to a specified destination.
    ///
    /// # Arguments
    ///
    /// * `destination_addr` - A `UnifiedAddress` representing the address of the destination to connect to.
    /// * `destination_type` - A `DestinationType` indicating the type of the destination (TCP or UDP).
    ///
    /// # Returns
    ///
    /// A `Result` which is either:
    /// * `Ok(ProxyConnection<ProxyFramedReadWrite<'a>>)` - A successfully established connection wrapped in a `ProxyConnection`.
    /// * `Err(Error)` - An error if the connection could not be established, detailing the failure.
    ///
    /// # Errors
    ///
    /// This function will return an `Error` if:
    /// * The conversion of `connect_destination_request` into bytes fails.
    /// * Sending the connection request through `proxy_framed` fails.
    /// * Receiving the response from the proxy times out or the connection is exhausted.
    /// * The destination responds with a failure message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use your_crate::DestinationType;
    /// # use your_crate::UnifiedAddress;
    /// # use your_crate::YourStruct; // Replace YourStruct with the actual struct name that contains this method
    /// # use your_crate::Error; // Assuming Error is defined in your crate
    /// # use futures::executor::block_on;
    /// # let destination_addr = UnifiedAddress::new(); // Example initialization
    /// # let your_struct_instance = YourStruct::new(); // Example initialization
    /// # block_on(async {
    /// match your_struct_instance.connect_destination(destination_addr, DestinationType::Tcp).await {
    ///     Ok(connection) => println!("Successfully connected: {:?}", connection),
    ///     Err(e) => eprintln!("Failed to connect: {}", e),
    /// }
    /// # });
    /// ```
    ///
    /// # Notes
    ///
    /// - The `self.state` must be a mutable reference to `ProxyFramedReadWrite` for sending and receiving data.
    /// - The `ConnectDestinationRequest` and `ConnectDestinationResponse` are expected to implement `TryFrom<&[u8]>` and `TryInto<Vec<u8>>` respectively, to handle serialization and deserialization.
    ///
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
        proxy_framed.send(&connect_destination_request_bytes).await?;
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
