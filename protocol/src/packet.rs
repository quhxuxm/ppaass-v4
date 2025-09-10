use crate::address::UnifiedAddress;
use crate::{Error, Username};
use bincode::config::Configuration;
use bytes::{Bytes, BytesMut};
use serde::{Deserialize, Serialize};

/// Represents different types of encryption that can be applied to data.
///
/// # Variants
///
/// * `Plain` - Indicates that no encryption is used. Data is stored and transmitted in plain text.
/// * `Aes(Bytes)` - Specifies that AES (Advanced Encryption Standard) encryption is used. The `Bytes` parameter holds the encrypted data.
/// * `Blowfish(Bytes)` - Indicates the use of Blowfish encryption. Similar to `Aes`, the `Bytes` parameter here contains the data after being encrypted with the Blowfish algorithm.
///
/// # Examples
///
/// ```
/// use your_crate::Encryption;
/// use bytes::Bytes;
///
/// let plain_data = Encryption::Plain;
/// let aes_encrypted_data = Encryption::Aes(Bytes::from_static(b"encrypted data"));
/// let blowfish_encrypted_data = Encryption::Blowfish(Bytes::from_static(b"another encrypted data"));
/// ```
///
/// This enum is useful for scenarios where you need to handle or specify different encryption methods within your application, allowing for flexibility in security implementations.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Encryption {
    /// No encryption
    Plain,
    /// Aes encryption
    Aes(Bytes),
    /// Blowfish encryption
    Blowfish(Bytes),
}

/// Represents a request for initiating a handshake in the system.
///
/// This struct is used to encapsulate the necessary information required
/// to start a secure communication, including the user's username and
/// the preferred method of encryption.
///
/// # Fields
///
/// * `username` - A `String` that holds the unique identifier of the user
///   attempting to initiate the handshake.
/// * `encryption` - An `Encryption` enum value indicating the type of
///   encryption the client prefers or is capable of using.
///
/// # Examples
///
/// ```
/// use your_crate::handshake::HandshakeRequest;
/// use your_crate::encryption::Encryption;
///
/// let request = HandshakeRequest {
///     username: "user123".to_string(),
///     encryption: Encryption::Aes256,
/// };
/// ```
///
/// # Derives
///
/// - `Debug`: For formatted output, useful for debugging.
/// - `Serialize`: To convert the struct into a format that can be easily
///   transmitted over the network or stored (e.g., JSON).
/// - `Deserialize`: To reconstruct the struct from a serialized format,
///   enabling easy data exchange and storage.
///
#[derive(Debug, Serialize, Deserialize)]
pub struct HandshakeRequest {
    pub username: Username,
    pub encryption: Encryption,
}

impl TryFrom<Bytes> for HandshakeRequest {
    type Error = Error;
    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        let (result, _) = bincode::serde::decode_from_slice::<HandshakeRequest, Configuration>(
            &value,
            bincode::config::standard(),
        )?;
        Ok(result)
    }
}

impl TryFrom<BytesMut> for HandshakeRequest {
    type Error = Error;
    fn try_from(value: BytesMut) -> Result<Self, Self::Error> {
        Self::try_from(value.freeze())
    }
}

impl TryFrom<HandshakeRequest> for Vec<u8> {
    type Error = Error;
    fn try_from(value: HandshakeRequest) -> Result<Self, Self::Error> {
        let result = bincode::serde::encode_to_vec(value, bincode::config::standard())?;
        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HandshakeResponse {
    pub encryption: Encryption,
}

impl TryFrom<Bytes> for HandshakeResponse {
    type Error = Error;
    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        let (result, _) = bincode::serde::decode_from_slice::<HandshakeResponse, Configuration>(
            &value,
            bincode::config::standard(),
        )?;
        Ok(result)
    }
}

impl TryFrom<BytesMut> for HandshakeResponse {
    type Error = Error;
    fn try_from(value: BytesMut) -> Result<Self, Self::Error> {
        Self::try_from(value.freeze())
    }
}

impl TryFrom<HandshakeResponse> for Vec<u8> {
    type Error = Error;
    fn try_from(value: HandshakeResponse) -> Result<Self, Self::Error> {
        let result = bincode::serde::encode_to_vec(value, bincode::config::standard())?;
        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConnectDestinationRequest {
    /// Connect the TCP destination
    Tcp(UnifiedAddress),
    /// Connect the UDP destination
    Udp(UnifiedAddress),
}

impl TryFrom<Bytes> for ConnectDestinationRequest {
    type Error = Error;
    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        let (result, _) = bincode::serde::decode_from_slice::<
            ConnectDestinationRequest,
            Configuration,
        >(&value, bincode::config::standard())?;
        Ok(result)
    }
}

impl TryFrom<BytesMut> for ConnectDestinationRequest {
    type Error = Error;
    fn try_from(value: BytesMut) -> Result<Self, Self::Error> {
        Self::try_from(value.freeze())
    }
}

impl TryFrom<ConnectDestinationRequest> for Vec<u8> {
    type Error = Error;
    fn try_from(value: ConnectDestinationRequest) -> Result<Self, Self::Error> {
        let result = bincode::serde::encode_to_vec(value, bincode::config::standard())?;
        Ok(result)
    }
}

/// Represents the response from a connection attempt to a destination.
///
/// This enum can have one of two values:
/// - `Success`: Indicates that the connection to the destination was successful.
/// - `Fail`: Indicates that the connection to the destination failed.
///
/// # Examples
///
/// ```
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, Serialize, Deserialize)]
/// pub enum ConnectDestinationResponse {
///     /// Connect to destination success
///     Success,
///     /// Connect to destination fail
///     Fail,
/// }
///
/// let response = ConnectDestinationResponse::Success;
/// println!("{:?}", response); // Outputs: Success
/// ```
///
#[derive(Debug, Serialize, Deserialize)]
pub enum ConnectDestinationResponse {
    /// Connect to destination success
    Success,
    /// Connect to destination fail
    Fail,
}

impl TryFrom<Bytes> for ConnectDestinationResponse {
    type Error = Error;
    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        let (result, _) = bincode::serde::decode_from_slice::<
            ConnectDestinationResponse,
            Configuration,
        >(&value, bincode::config::standard())?;
        Ok(result)
    }
}

impl TryFrom<BytesMut> for ConnectDestinationResponse {
    type Error = Error;
    fn try_from(value: BytesMut) -> Result<Self, Self::Error> {
        Self::try_from(value.freeze())
    }
}

impl TryFrom<ConnectDestinationResponse> for Vec<u8> {
    type Error = Error;
    fn try_from(value: ConnectDestinationResponse) -> Result<Self, Self::Error> {
        let result = bincode::serde::encode_to_vec(value, bincode::config::standard())?;
        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Relay {
    /// Relay TCP data
    Tcp(Bytes),
    /// Relay UDP data
    Udp {
        src_addr: UnifiedAddress,
        dst_addr: UnifiedAddress,
        payload: Bytes,
    },
}

impl TryFrom<Bytes> for Relay {
    type Error = Error;
    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        let (result, _) = bincode::serde::decode_from_slice::<Relay, Configuration>(
            &value,
            bincode::config::standard(),
        )?;
        Ok(result)
    }
}

impl TryFrom<BytesMut> for Relay {
    type Error = Error;
    fn try_from(value: BytesMut) -> Result<Self, Self::Error> {
        Self::try_from(value.freeze())
    }
}

impl TryFrom<Relay> for Vec<u8> {
    type Error = Error;
    fn try_from(value: Relay) -> Result<Self, Self::Error> {
        let result = bincode::serde::encode_to_vec(value, bincode::config::standard())?;
        Ok(result)
    }
}
