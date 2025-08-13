use crate::command::CommandArgs;
use clap::Parser;
use common::config::CommonConfig;
use common::UserConfig;
use core::panic;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::sync::OnceLock;

/// The default configuration file patch
const DEFAULT_CONFIG_FILE: &str = "./resources/agent.toml";
/// The global configuration object
static CONFIG: OnceLock<Config> = OnceLock::new();

/// Get the configuration
pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        let command_line = CommandArgs::parse();
        let config_content = match &command_line.config_file_path {
            None => read_to_string(DEFAULT_CONFIG_FILE).unwrap_or_else(|_| {
                panic!(
                    "Fail to read agent configuration file content from: {:?}",
                    DEFAULT_CONFIG_FILE
                )
            }),
            Some(path) => read_to_string(path).unwrap_or_else(|_| {
                panic!(
                    "Fail to read agent configuration file content from: {:?}",
                    path
                )
            }),
        };
        let mut config = toml::from_str::<Config>(&config_content)
            .expect("Fail to initialize agent configuration");
        config.merge_command_args(command_line);
        config
    })
}

/// The configuration object
#[derive(
    Serialize,
    Deserialize,
    Debug,
)]
pub struct Config {
    #[serde(flatten)]
    common: CommonConfig,
    proxy_connect_timeout: u64,
    username: String,
}

impl Config {
    pub fn proxy_connect_timeout(&self) -> u64 {
        self.proxy_connect_timeout
    }
    pub fn common(&self) -> &CommonConfig {
        &self.common
    }
    pub fn merge_command_args(&mut self, command: CommandArgs) {
        if let Some(listening_address) = command.listening_address {
            self.common.listening_address = listening_address;
        }
        if let Some(worker_threads) = command.worker_threads {
            self.common.worker_threads = worker_threads;
        }
        if let Some(log_directory) = command.log_directory {
            self.common.log_directory = log_directory;
        }
        if let Some(max_log_level) = command.max_log_level {
            self.common.max_log_level = max_log_level;
        }
        if let Some(user_repo_directory) = command.user_repo_directory {
            self.common.user_repo_directory = user_repo_directory;
        }
        if let Some(user_repo_refresh_interval) = command.user_repo_refresh_interval {
            self.common.user_repo_refresh_interval = user_repo_refresh_interval;
        }
        if let Some(username) = command.username {
            self.username = username;
        }
    }
}

impl UserConfig for Config {
    fn username(&self) -> &str {
        &self.username
    }
}

