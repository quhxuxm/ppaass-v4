use crate::command::CommandArgs;
use clap::Parser;
use common::config::CommonConfig;
use common::{FsUserRepoConfig, UserConfig, UserRepoConfig};
use core::panic;
use protocol::Username;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const DEFAULT_CONFIG_FILE: &str = "./resources/proxy.toml";
static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        let command_line = CommandArgs::parse();
        let config_content = match &command_line.config_file_path {
            None => read_to_string(DEFAULT_CONFIG_FILE).unwrap_or_else(|_| {
                panic!(
                    "Fail to read proxy configuration file content from: {:?}",
                    DEFAULT_CONFIG_FILE
                )
            }),
            Some(path) => read_to_string(path).unwrap_or_else(|_| {
                panic!(
                    "Fail to read proxy configuration file content from: {:?}",
                    path
                )
            }),
        };
        let mut config = toml::from_str::<Config>(&config_content)
            .expect("Fail to initialize proxy configuration");
        config.merge_command_args(command_line);
        config
    })
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForwardConfig {
    proxy_connect_timeout: u64,
    user_info_file_name: String,
    user_info_private_key_file_name: String,
    user_info_public_key_file_name: String,
    user_repo_directory: PathBuf,
    user_repo_refresh_interval: u64,
    username: Username,
}

impl ForwardConfig {
    pub fn proxy_connect_timeout(&self) -> u64 {
        self.proxy_connect_timeout
    }
}

impl UserConfig for ForwardConfig {
    fn username(&self) -> &Username {
        &self.username
    }
}

impl UserRepoConfig for ForwardConfig {
    fn refresh_interval_sec(&self) -> u64 {
        self.user_repo_refresh_interval
    }
}

impl FsUserRepoConfig for ForwardConfig {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    common_config: CommonConfig,
    destination_connect_timeout: u64,
    forward: Option<ForwardConfig>,
}

impl Config {
    pub fn destination_connect_timeout(&self) -> u64 {
        self.destination_connect_timeout
    }
    pub fn common(&self) -> &CommonConfig {
        &self.common_config
    }
    pub fn merge_command_args(&mut self, command: CommandArgs) {
        if let Some(listening_address) = command.listening_address {
            self.common_config.listening_address = listening_address;
        }
        if let Some(worker_threads) = command.worker_threads {
            self.common_config.worker_threads = worker_threads;
        }
        if let Some(log_directory) = command.log_directory {
            self.common_config.log_directory = log_directory;
        }
        if let Some(max_log_level) = command.max_log_level {
            self.common_config.max_log_level = max_log_level;
        }
        if let Some(user_repo_directory) = command.user_repo_directory {
            self.common_config.user_repo_directory = user_repo_directory;
        }
        if let Some(user_repo_refresh_interval) = command.user_repo_refresh_interval {
            self.common_config.user_repo_refresh_interval = user_repo_refresh_interval;
        }
    }
    pub fn forward(&self) -> Option<&ForwardConfig> {
        self.forward.as_ref()
    }
}
