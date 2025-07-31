use serde::{Deserialize, Serialize};
use std::{net::IpAddr, str::FromStr};

/// Application configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigurationFile {
    pub port: u16,
    pub address: IpAddr,
    pub database_url: String,
}

impl ConfigurationFile {
    pub fn default() -> Self {
        ConfigurationFile {
            port: 3000,
            address: IpAddr::from_str("0.0.0.0").unwrap(),
            database_url: "random-words.db".to_string(),
        }
    }
}
