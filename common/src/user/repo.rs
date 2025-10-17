use crate::Error;
use crate::config::FsUserRepoConfig;
use crate::user::User;
use crate::user::UserRepository;
use ppaass_crypto::RsaCrypto;
use ppaass_protocol::Username;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;
use tracing::error;

#[derive(Debug)]
pub struct FileSystemUserRepository<U, C>
where
    U: User + Send + Sync + DeserializeOwned + 'static,
    C: FsUserRepoConfig + Send + Sync + 'static,
{
    storage: HashMap<Username, U>,
    _config_mark: PhantomData<C>,
}

impl<U, C> FileSystemUserRepository<U, C>
where
    U: User + Send + Sync + DeserializeOwned + 'static,
    C: FsUserRepoConfig + Send + Sync + 'static,
{
    fn fill_storage(config: &C, storage: &mut HashMap<Username, U>) -> Result<(), Error> {
        let user_repo_directory_path = config.user_repo_directory();
        let mut user_repo_directory = std::fs::read_dir(user_repo_directory_path)?;
        while let Some(Ok(sub_entry)) = user_repo_directory.next() {
            let file_type = match sub_entry.file_type() {
                Ok(file_type) => file_type,
                Err(e) => {
                    error!(
                        "Fail to read sub entry from user user directory [{user_repo_directory_path:?}] because of error: {e:?}"
                    );
                    continue;
                }
            };
            if !file_type.is_dir() {
                continue;
            }
            let file_name = sub_entry.file_name();
            let file_name = match file_name.to_str() {
                Some(file_name) => file_name,
                None => {
                    continue;
                }
            };
            if file_name.starts_with("\\.") {
                continue;
            }
            let user_dir_path = sub_entry.path();
            let public_key_file_path = user_dir_path.join(config.public_key_file_name());
            let public_key_file = match std::fs::File::open(public_key_file_path) {
                Ok(public_key_file) => public_key_file,
                Err(e) => {
                    error!("Fail to read public key file: {e:?}");
                    continue;
                }
            };
            let private_key_file_path = user_dir_path.join(config.private_key_file_name());
            let private_key_file = match std::fs::File::open(private_key_file_path) {
                Ok(private_key_file) => private_key_file,
                Err(e) => {
                    error!("Fail to read private key file: {e:?}");
                    continue;
                }
            };
            let user_rsa_crypto = match RsaCrypto::new(public_key_file, private_key_file) {
                Ok(user_rsa_crypto) => user_rsa_crypto,
                Err(e) => {
                    error!("Fail to create user rsa crypto: {e:?}");
                    continue;
                }
            };
            let user_info_file_path = user_dir_path.join(config.user_info_file_name());
            let user_info_file_content = match std::fs::read_to_string(&user_info_file_path) {
                Ok(content) => content,
                Err(e) => {
                    error!("Fail to read user info file content: {e:?}");
                    continue;
                }
            };
            let mut user_info = match toml::from_str::<U>(&user_info_file_content) {
                Ok(user_info) => user_info,
                Err(e) => {
                    error!("Fail to deserialize the user info: {e:?}");
                    continue;
                }
            };
            user_info.set_rsa_crypto(user_rsa_crypto);
            storage.insert(user_info.username().clone(), user_info);
        }
        Ok(())
    }
}

impl<U, C> UserRepository for FileSystemUserRepository<U, C>
where
    U: User + Send + Sync + DeserializeOwned + 'static,
    C: FsUserRepoConfig + Send + Sync + 'static,
{
    type UserInfoType = U;
    type UserRepoConfigType = C;
    fn new<T>(config: T) -> Result<Self, Error>
    where
        T: Deref<Target = Self::UserRepoConfigType> + Send + Sync + 'static,
    {
        let mut storage = HashMap::new();
        if let Err(e) = Self::fill_storage(&config, &mut storage) {
            error!("Failed to fill user repository storage: {}", e);
        };
        Ok(Self {
            storage,
            _config_mark: Default::default(),
        })
    }

    fn find_user(&self, username: &Username) -> Option<&Self::UserInfoType> {
        let user_info = self.storage.get(username)?;
        Some(user_info)
    }

    fn save_user(&mut self, user: Self::UserInfoType) {
        self.storage.insert(user.username().to_owned(), user);
    }
}
