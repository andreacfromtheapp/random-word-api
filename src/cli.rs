// API command line interface
use clap::{Args, Parser};
use std::net::IpAddr;
use std::ops::RangeInclusive;
use std::path::PathBuf;

/// Define the OS port range
const PORT_RANGE: RangeInclusive<usize> = 1..=65535;

/// Command line interface arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    /// Configuration arguments group. Mutually exclusive with command line arguments
    pub cfg: Config,

    #[command(flatten)]
    /// Command line argument groups. Mutually exclusive with configuration arguments
    pub args: Arguments,
}

/// Configuration arguments. Mutually exclusive with command line arguments
#[derive(Args, Debug)]
#[group(id = "cfg", multiple = false, conflicts_with = "args")]
pub struct Config {
    /// Configuration file path
    #[arg(short, long, default_value = "./config.toml")]
    pub config: Option<PathBuf>,

    /// Environment file path
    #[arg(short, long)]
    pub env_file: Option<PathBuf>,
}

/// Command line arguments. Mutually exclusive with configuration arguments
#[derive(Args, Debug)]
#[group(id = "args", multiple = true, conflicts_with = "cfg")]
pub struct Arguments {
    /// API IP address to bind
    #[arg(short, long, default_value = "0.0.0.0")]
    pub address: IpAddr,

    /// API port number to bind
    #[arg(short, long, default_value_t = 3000, value_parser = port_in_range)]
    pub port: u16,

    /// API database connection URL
    #[arg(short, long, default_value = "sqlite:random-words.db")]
    pub database_url: String,
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
            "port not in range. min: {} max: {}",
            PORT_RANGE.start(),
            PORT_RANGE.end()
        ))
    }
}
