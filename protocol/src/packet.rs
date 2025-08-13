use crate::address::UnifiedAddress;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Encryption {
    Plain,
    Aes(Bytes),
    Blowfish(Bytes),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientHandshake {
    pub username: String,
    pub encryption: Encryption,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerHandshake {
    pub encryption: Encryption,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientSetupDestination {
    Tcp(UnifiedAddress),
    Udp(UnifiedAddress),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerSetupDestination {
    Success,
    Fail,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Relay {
    Tcp(Bytes),
    Udp {
        src_addr: UnifiedAddress,
        dst_addr: UnifiedAddress,
        payload: Bytes,
    },
}
