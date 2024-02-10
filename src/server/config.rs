use std::path::Path;
use serde::*;

use super::error::ServerConfigError;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub(crate) struct ServerConfig {
    pub port: u32
}

impl ServerConfig {
    pub fn load_from_file<T: AsRef<Path>>(path: T) -> Result<Self, ServerConfigError> {
        let file = std::fs::read_to_string(path).unwrap();
        match serde_json::from_str(&file) {
            Ok(v) => Ok(v),
            Err(_) => Err(ServerConfigError::FailedToLoadConfig),
        }
    }
}