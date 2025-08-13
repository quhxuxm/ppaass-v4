use crate::address::UnifiedAddress;
use bytes::Bytes;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct HandshakeResponse {
    pub encryption: Encryption,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConnectDestinationRequest {
    /// Connect the TCP destination
    Tcp(UnifiedAddress),
    /// Connect the UDP destination
    Udp(UnifiedAddress),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConnectDestinationResponse {
    /// Connect to destination success
    Success,
    /// Connect to destination fail
    Fail,
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
