mod address;
pub mod error;

pub use address::*;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum Encryption {
    Aes(Vec<u8>),
    Blowfish(Vec<u8>),
    Plain,
}

pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string().replace("-", "")
}

pub fn random_bytes<const N: usize>() -> [u8; N] {
    rand::random::<[u8; N]>()
}
