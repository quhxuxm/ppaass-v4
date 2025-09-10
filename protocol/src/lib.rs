mod address;
mod error;
mod packet;

pub use address::*;
pub use error::*;
pub use packet::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct Username(pub String);
