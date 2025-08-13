use crate::address::UnifiedAddress;
use crate::Error;
use bincode::config::Configuration;
use bytes::{Bytes, BytesMut};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Encryption {
    /// No encryption
    Plain,
    /// Aes encryption
    Aes(Bytes),
    /// Blowfish encryption
    Blowfish(Bytes),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HandshakeRequest {
    pub username: String,
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
        let result =
            bincode::serde::encode_to_vec(value, bincode::config::standard())?;
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
        let result =
            bincode::serde::encode_to_vec(value, bincode::config::standard())?;
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
        let (result, _) = bincode::serde::decode_from_slice::<ConnectDestinationRequest, Configuration>(
            &value,
            bincode::config::standard(),
        )?;
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
        let result =
            bincode::serde::encode_to_vec(value, bincode::config::standard())?;
        Ok(result)
    }
}

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
        let (result, _) = bincode::serde::decode_from_slice::<ConnectDestinationResponse, Configuration>(
            &value,
            bincode::config::standard(),
        )?;
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
        let result =
            bincode::serde::encode_to_vec(value, bincode::config::standard())?;
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
        let result =
            bincode::serde::encode_to_vec(value, bincode::config::standard())?;
        Ok(result)
    }
}
