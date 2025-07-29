use clap::{Args, Parser};
use std::net::IpAddr;
use std::ops::RangeInclusive;

use crate::word::Environment;

/// Constant to define the POSIX port range
const PORT_RANGE: RangeInclusive<usize> = 1..=65535;

/// Arguments for clap
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    /// Configuration arguments. Can only use one of them
    pub cfg: Config,

    /// The IPv4 address to bind the API to
    #[arg(short, long, default_value = "0.0.0.0", env = "BIND_ADDR")]
    pub address: IpAddr,

    /// The port number to bind the API to
    #[arg(short, long, default_value_t = 3000, env = "BIND_PORT", value_parser = port_in_range)]
    pub port: u16,

    /// The database connection URL
    #[arg(
        short,
        long,
        default_value = "sqlite:random-words.db",
        env = "DATABASE_URL"
    )]
    pub database_url: String,
}

/// Configuration argument. Can only use one of them
#[derive(Args, Debug)]
#[group(required = false, multiple = false)]
pub struct Config {
    /// Configuration file path
    #[arg(long, default_value = "$HOME/.config/random-api/config.toml")]
    pub config: String,

    /// Environment file path
    #[arg(long, default_value = ".env")]
    pub env_file: String,

    /// Environment: Development | Test | Production
    #[arg(long, default_value_t = Environment::Development, value_enum)]
    pub env: Environment,
}

/// Validate that the port number is within the OS ports range
fn port_in_range(s: &str) -> Result<u16, String> {
    let port: usize = s
        .parse()
        .map_err(|_| format!("`{s}` isn't a port number"))?;

    if PORT_RANGE.contains(&port) {
        Ok(port as u16)
    } else {
        Err(format!(
            "port not in range '{} - {}'",
            PORT_RANGE.start(),
            PORT_RANGE.end()
        ))
    }
}
