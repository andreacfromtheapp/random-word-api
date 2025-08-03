// API command line interface
use clap::{Args, Parser, Subcommand};
use std::net::IpAddr;
use std::ops::RangeInclusive;
use std::path::PathBuf;

/// Define the OS port range
const PORT_RANGE: RangeInclusive<usize> = 1..=65535;

/// Command line interface
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    /// Configuration options
    pub cfg: Config,

    #[command(flatten)]
    /// Argument options
    pub arg: Arguments,

    /// Commands
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Command line subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate a configuration file with default values
    Setup {
        /// Configuration file path
        #[arg(short, long, value_name = "FILE")]
        create_config: Option<PathBuf>,
    },
}

/// Configuration arguments
#[derive(Args, Debug)]
#[group(id = "cfg", required = false, multiple = false, conflicts_with = "arg")]
pub struct Config {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Environment file path
    #[arg(short, long, value_name = "FILE")]
    pub env_file: Option<PathBuf>,
}

/// Command line arguments
#[derive(Args, Debug)]
#[group(id = "arg", multiple = true, conflicts_with = "cfg")]
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
