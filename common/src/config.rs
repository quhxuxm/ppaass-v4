use ppaass_protocol::Username;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

/// A trait that defines methods for accessing server configuration details.
///
/// This trait is designed to be implemented by types that need to provide
/// information about the server's setup, such as the address it listens on,
/// the maximum number of client connections it can handle, and the number of
/// worker threads it uses. Implementing this trait allows for a consistent
/// interface to access these properties across different server implementations.
///
/// # Methods
///
/// * `listening_address` - Returns the address (IP and port) the server is listening on.
/// * `client_max_connections` - Specifies the maximum number of simultaneous client connections allowed.
/// * `worker_threads` - Indicates the maximum number of worker threads the server will utilize.
///
pub trait ServerConfig {
    /// Returns the socket address that this instance is listening on.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::SocketAddr;
    ///
    /// // Assuming `my_instance` is an instance of a struct implementing this method
    /// let addr: SocketAddr = my_instance.listening_address();
    /// println!("Listening on: {}", addr);
    /// ```
    ///
    /// # Returns
    ///
    /// * `SocketAddr` - The socket address (IP and port) where the instance is currently listening for incoming connections.
    fn listening_address(&self) -> SocketAddr;
    /// Returns the maximum number of connections allowed for a client.
    ///
    /// # Returns
    ///
    /// * `usize` - The maximum number of connections as an unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// let max_connections = some_client.client_max_connections();
    /// println!("Maximum connections: {}", max_connections);
    /// ```
    ///
    fn client_max_connections(&self) -> usize;
    /// Returns the number of worker threads currently configured for the system or process.
    ///
    /// # Returns
    ///
    /// * `usize` - The number of worker threads.
    ///
    /// # Examples
    ///
    /// ```
    /// let system = System::new();
    /// let num_threads = system.worker_threads();
    /// println!("Number of worker threads: {}", num_threads);
    /// ```
    ///
    fn worker_threads(&self) -> usize;
    /// Returns a reference to the path of the log directory.
    ///
    /// # Examples
    ///
    /// ```
    /// let log_path = some_instance.log_directory();
    /// println!("The log directory is: {:?}", log_path);
    /// ```
    fn log_directory(&self) -> &Path;
    /// Returns the prefix used for logging messages.
    ///
    /// This method is useful when you need to prepend a specific identifier or
    /// tag to log messages, making it easier to filter or search through logs.
    /// The returned string can be used directly in conjunction with logging
    /// frameworks or custom logging implementations.
    ///
    /// # Examples
    ///
    /// ```
    /// let prefix = some_object.log_name_prefix();
    /// println!("{}: This is a log message", prefix);
    /// ```
    ///
    /// # Returns
    ///
    /// A string slice that represents the prefix for log messages.
    fn log_name_prefix(&self) -> &str;
    /// Returns the maximum log level as a string slice.
    ///
    /// This method is used to determine the highest level of logging
    /// that is currently enabled. It returns a string representation
    /// of the log level, which can be used for filtering log messages
    /// or for informational purposes.
    ///
    /// # Examples
    ///
    /// ```
    /// let log_level = some_logger.max_log_level();
    /// println!("The current max log level is: {}", log_level);
    /// ```
    ///
    /// # Returns
    ///
    /// A string slice representing the maximum log level.
    ///
    fn max_log_level(&self) -> &str;
}

///
/// A trait that defines a method to retrieve the refresh interval in seconds from
/// a configuration associated with a user repository. This is typically used in
/// scenarios where periodic updates or refreshes are required, and the timing of
/// these operations needs to be configurable.
///
/// # Methods
///
/// * `refresh_interval_sec` - Retrieves the current refresh interval setting in seconds.
///
/// Implementers of this trait should ensure that the `refresh_interval_sec` method
/// accurately reflects the desired refresh rate for their specific use case, allowing
/// for flexibility in how often certain operations, such as data synchronization or
/// cache refreshing, are performed.
///
/// For more details on implementing and using this trait, refer to the example and
/// method documentation provided within.
///
pub trait UserRepoConfig {
    /// Returns the refresh interval in seconds.
    ///
    /// This method provides the current setting for how often a refresh operation
    /// should be performed, expressed in seconds. It is useful for determining the
    /// frequency of periodic tasks or updates within the application.
    ///
    /// # Examples
    ///
    /// ```
    /// let interval = obj.refresh_interval_sec();
    /// println!("The refresh interval is set to {} seconds.", interval);
    /// ```
    ///
    /// # Returns
    ///
    /// * `u64` - The refresh interval in seconds.
    fn refresh_interval_sec(&self) -> u64;
}

/// A trait that can be implemented by any struct or type that has a username.
/// This trait provides a method to retrieve the username of the implementing type.
///
/// # Methods
///
/// * `username` - Returns the username associated with the current instance.
///
/// # Examples
///
/// ```
/// let user = User { name: "JohnDoe" };
/// assert_eq!(user.username(), "JohnDoe");
/// ```
///
/// Implementing this trait allows for a consistent way to access the username across different types,
/// which can be particularly useful in applications where multiple types of users or entities
/// each have their own unique identifier (username).
///
/// The example provided shows a simple use case where a `User` struct, having a field `name`,
/// implements the `WithUsernameConfig` trait. The `username` method is then used to get the value
/// of the `name` field, demonstrating how the trait can be applied.
///
pub trait UserConfig {
    /// Returns the username associated with the current instance.
    ///
    /// # Examples
    ///
    /// ```
    /// let user = User { name: "JohnDoe" };
    /// assert_eq!(user.username(), "JohnDoe");
    /// ```
    ///
    fn username(&self) -> &Username;
}

/// A trait that extends `WithUserRepositoryConfig` to provide file system-specific
/// configuration for a user repository. This trait is designed to be implemented by
/// types that need to specify the directory and file names used for storing user
/// information, public keys, and private keys on the file system.
///
/// # Required Methods
///
/// - `user_repo_directory(&self) -> &Path`: Returns a reference to the path where
///   the user repository files are stored.
///
/// - `public_key_file_name(&self) -> &str`: Returns the name of the file that
///   contains the public key for the user.
///
/// - `private_key_file_name(&self) -> &str`: Returns the name of the file that
///   contains the private key for the user.
///
/// - `user_info_file_name(&self) -> &str`: Returns the name of the file that
///   contains additional user information.
///
/// Implementing this trait allows for flexible configuration of the file system
/// paths and filenames used in a user management system, making it easier to
/// adapt to different storage layouts or requirements.
///
pub trait FsUserRepoConfig: UserRepoConfig {
    /// Returns a reference to the path of the user's repository directory.
    ///
    /// This method provides access to the specific directory where the user's
    /// repositories or related files are stored. It is useful for operations
    /// that need to interact with the file system at the location designated
    /// for storing user-specific data.
    ///
    /// # Examples
    ///
    /// ```
    /// let repo_dir = some_instance.user_repo_directory();
    /// println!("User's repository directory: {:?}", repo_dir);
    /// ```
    ///
    /// # Returns
    ///
    /// A `&Path` representing the user's repository directory.
    ///
    fn user_repo_directory(&self) -> &Path;
    /// Returns the file name of the public key associated with the current instance.
    ///
    /// # Returns
    /// A string slice (`&str`) representing the file name of the public key.
    ///
    fn public_key_file_name(&self) -> &str;
    /// Returns the file name for the private key associated with the current instance.
    ///
    /// # Returns
    /// A string slice that represents the file name of the private key.
    ///
    fn private_key_file_name(&self) -> &str;
    ///
    /// Returns the file name used for storing user information.
    ///
    /// This method provides a string slice that represents the name of the file
    /// where user information is stored. It's useful for file operations or when
    /// the file name needs to be referenced in the system.
    ///
    /// # Examples
    ///
    /// ```
    /// let user_info = UserInfo::new();
    /// assert_eq!(user_info.user_info_file_name(), "user_data.txt");
    /// ```
    ///
    /// Note: The actual file name returned may vary based on the implementation.
    ///
    fn user_info_file_name(&self) -> &str;
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
)]
pub struct CommonConfig {
    pub client_max_connections: usize,
    pub listening_address: SocketAddr,
    pub log_directory: PathBuf,
    pub log_name_prefix: String,
    pub max_log_level: String,
    pub user_info_file_name: String,
    pub user_info_private_key_file_name: String,
    pub user_info_public_key_file_name: String,
    pub user_repo_directory: PathBuf,
    pub user_repo_refresh_interval: u64,
    pub worker_threads: usize,
}

impl ServerConfig for CommonConfig {
    fn listening_address(&self) -> SocketAddr {
        self.listening_address
    }
    fn client_max_connections(&self) -> usize {
        self.client_max_connections
    }
    fn worker_threads(&self) -> usize {
        self.worker_threads
    }
    fn log_directory(&self) -> &Path {
        &self.log_directory
    }
    fn log_name_prefix(&self) -> &str {
        &self.log_name_prefix
    }
    fn max_log_level(&self) -> &str {
        &self.max_log_level
    }
}

impl UserRepoConfig for CommonConfig {
    fn refresh_interval_sec(&self) -> u64 {
        self.user_repo_refresh_interval
    }
}

impl FsUserRepoConfig for CommonConfig {
    fn user_repo_directory(&self) -> &Path {
        &self.user_repo_directory
    }
    fn public_key_file_name(&self) -> &str {
        &self.user_info_public_key_file_name
    }
    fn private_key_file_name(&self) -> &str {
        &self.user_info_private_key_file_name
    }
    fn user_info_file_name(&self) -> &str {
        &self.user_info_file_name
    }
}
