mod codec;
pub mod config;
mod error;
pub mod log;
pub mod proxy;
mod runtime;
mod server;
pub mod user;

pub use codec::SecureLengthDelimitedCodec;
pub use config::FsUserRepoConfig;
pub use config::ServerConfig;
pub use config::UserConfig;
pub use config::UserRepoConfig;
pub use error::Error;
use ppaass_crypto::{generate_aes_encryption_token, generate_blowfish_encryption_token, RsaCrypto};
use ppaass_protocol::Encryption;
use rand::random;
pub use runtime::build_server_runtime;
pub use server::start_server;
pub use server::ServerGuard;
pub use server::ServerState;
use std::borrow::Cow;
use std::sync::Arc;
use std::sync::LazyLock;

static HANDSHAKE_ENCRYPTION: LazyLock<Arc<Encryption>> = LazyLock::new(|| {
    Arc::new(Encryption::Blowfish({
        b"1212398347384737434783748347387438743742982332672763272320119203".to_vec().into()
    }))
});

pub fn get_handshake_encryption<'a>() -> &'a Encryption {
    &HANDSHAKE_ENCRYPTION
}

/// Randomly generate a raw encryption
#[inline(always)]
pub fn random_generate_encryption() -> Encryption {
    let random_number = random::<u64>();
    if random_number % 2 == 0 {
        Encryption::Aes(generate_aes_encryption_token())
    } else {
        Encryption::Blowfish(generate_blowfish_encryption_token())
    }
}

#[inline(always)]
pub fn rsa_encrypt_encryption<'a>(
    raw_encryption: &'a Encryption,
    rsa_crypto: &RsaCrypto,
) -> Result<Cow<'a, Encryption>, Error> {
    match raw_encryption {
        Encryption::Plain => Ok(Cow::Borrowed(raw_encryption)),
        Encryption::Aes(token) => {
            let encrypted_token = rsa_crypto.encrypt(token)?;
            Ok(Cow::Owned(Encryption::Aes(encrypted_token)))
        }
        Encryption::Blowfish(token) => {
            let encrypted_token = rsa_crypto.encrypt(token)?;
            Ok(Cow::Owned(Encryption::Blowfish(encrypted_token)))
        }
    }
}

#[inline(always)]
pub fn rsa_decrypt_encryption(
    encrypted_encryption: Encryption,
    rsa_crypto: &RsaCrypto,
) -> Result<Encryption, Error> {
    match encrypted_encryption {
        Encryption::Plain => Ok(encrypted_encryption),
        Encryption::Aes(token) => {
            let decrypted_token = rsa_crypto.decrypt(&token)?;
            Ok(Encryption::Aes(decrypted_token))
        }
        Encryption::Blowfish(token) => {
            let decrypted_token = rsa_crypto.decrypt(&token)?;
            Ok(Encryption::Blowfish(decrypted_token))
        }
    }
}
