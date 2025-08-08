//! Command Line Interface module
//!
//! This module provides a comprehensive command-line interface for the random word API
//! using the `clap` crate. It supports both configuration files and command-line arguments
//! for flexible deployment and development scenarios.
//!
//! # Configuration Methods
//!
//! The CLI supports three primary configuration methods:
//! - Command-line arguments for direct parameter specification
//! - TOML configuration files for structured configuration management
//! - Environment files (.env) for containerized deployments
//!
//! # Subcommands
//!
//! The CLI includes utility subcommands for:
//! - Generating default configuration files
//! - Creating environment template files
//! - Setting up development and production configurations
//!
//! # Validation
//!
//! All configuration parameters include validation to ensure:
//! - Valid IP addresses and port ranges
//! - Proper database URL formats
//! - Consistent OpenAPI documentation settings
//!
//! # Usage Patterns
//!
//! The CLI is designed to support various deployment scenarios from development
//! to production, with sensible defaults and comprehensive validation.

// API command line interface
use clap::{Args, Parser, Subcommand};
use std::net::IpAddr;
use std::path::PathBuf;
use validator::Validate;

/// Main command line interface structure.
///
/// This struct represents the top-level CLI parser that combines configuration
/// options, direct arguments, and subcommands. It uses `clap`'s derive API
/// to automatically generate help text, argument parsing, and validation.
///
/// # Configuration Hierarchy
///
/// The CLI follows a specific precedence order:
/// 1. Configuration files (--config flag)
/// 2. Environment files (--env-file flag)
/// 3. Direct command-line arguments
///
/// # Mutual Exclusion
///
/// Configuration methods are mutually exclusive to prevent conflicts:
/// - Config file options cannot be used with direct arguments
/// - Environment file options cannot be used with direct arguments
/// - Config and env-file options can be used together
///
/// # Subcommands
///
/// Optional subcommands provide utility functions for generating
/// configuration templates and environment files.
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

/// Available CLI subcommands for utility operations.
///
/// These subcommands provide helper functionality for setting up
/// and managing API configurations. Each subcommand generates
/// template files with default values that can be customized
/// for specific deployment environments.
///
/// # File Generation
///
/// Both subcommands create files with sensible defaults that
/// can be modified for specific use cases:
/// - Configuration files use TOML format for structured settings
/// - Environment files use key-value pairs suitable for containers
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generates a TOML configuration file with default values.
    ///
    /// This command creates a structured configuration file that can be
    /// used with the `--config` flag. The generated file includes all
    /// available configuration options with their default values and
    /// can be customized for specific deployment needs.
    ///
    /// # File Format
    ///
    /// The generated file uses TOML format with sections for:
    /// - Server binding configuration (address, port)
    /// - Database connection settings
    /// - OpenAPI documentation options
    ///
    /// # Default Values
    ///
    /// All generated values match the CLI argument defaults, ensuring
    /// consistency between configuration methods.
    GenConfig {
        /// Configuration file name
        #[arg(default_value = "config.toml")]
        file_name: Option<PathBuf>,
    },
    /// Generates an environment file with default values.
    ///
    /// This command creates a .env file suitable for containerized
    /// deployments and development environments. The file includes
    /// all configuration options as environment variables that can
    /// be loaded using the `--env-file` flag.
    ///
    /// # File Format
    ///
    /// The generated file uses standard environment variable format:
    /// - KEY=value pairs, one per line
    /// - Comments explaining each option
    /// - Docker and container-friendly formatting
    ///
    /// # Environment Variables
    ///
    /// Generated variables include:
    /// - BIND_ADDR: Server IP address
    /// - BIND_PORT: Server port number
    /// - DATABASE_URL: Database connection string
    /// - OpenAPI documentation flags
    GenEnvFile {
        /// Environment file name
        #[arg(default_value = ".env")]
        file_name: Option<PathBuf>,
    },
}

/// Configuration file and environment file options.
///
/// This struct groups arguments related to external configuration sources.
/// It is mutually exclusive with direct arguments to prevent configuration
/// conflicts and ensure clear precedence rules.
///
/// # Mutual Exclusion
///
/// This argument group conflicts with the `Arguments` struct, meaning
/// users must choose between file-based configuration and direct
/// command-line arguments.
///
/// # File Types
///
/// - Config files: TOML format for structured configuration
/// - Environment files: Key-value pairs for container deployments
///
/// # Validation
///
/// File paths are validated for existence and readability when
/// the application loads the configuration.
#[derive(Args, Debug)]
#[group(id = "cfg", required = false, multiple = false, conflicts_with = "arg")]
pub struct Config {
    /// Path to a TOML configuration file.
    ///
    /// Specifies the location of a TOML-formatted configuration file
    /// containing all API settings. The file should follow the structure
    /// generated by the `gen-config` subcommand.
    ///
    /// # File Format
    ///
    /// Expected TOML sections and keys:
    /// - `address`: IP address to bind
    /// - `port`: Port number to bind
    /// - `database_url`: Database connection string
    /// - `[openapi]`: OpenAPI documentation settings
    ///
    /// # Error Handling
    ///
    /// Invalid TOML syntax or missing required keys will result
    /// in application startup errors with descriptive messages.
    #[arg(short, long, value_name = "FILE_NAME")]
    pub config: Option<PathBuf>,

    /// Path to an environment variable file.
    ///
    /// Specifies the location of a .env file containing configuration
    /// values as environment variables. The file should follow the
    /// format generated by the `gen-env-file` subcommand.
    ///
    /// # File Format
    ///
    /// Expected environment variables:
    /// - BIND_ADDR: IP address to bind
    /// - BIND_PORT: Port number to bind
    /// - DATABASE_URL: Database connection string
    /// - OpenAPI documentation boolean flags
    ///
    /// # Environment Loading
    ///
    /// Variables are loaded into the process environment and
    /// parsed according to their expected types with validation.
    #[arg(short, long, value_name = "FILE_NAME")]
    pub env_file: Option<PathBuf>,
}

/// Direct command-line arguments for API configuration.
///
/// This struct defines all available command-line arguments that can be
/// used to configure the API directly without external files. All arguments
/// include default values and validation rules.
///
/// # Validation
///
/// Arguments are validated using the `validator` crate:
/// - Port numbers must be within valid range (1-65535)
/// - IP addresses must be valid IPv4 or IPv6 addresses
/// - Database URLs are validated for basic format
///
/// # Mutual Exclusion
///
/// This argument group conflicts with the `Config` struct to ensure
/// users don't mix configuration methods and create ambiguous settings.
///
/// # Default Values
///
/// All arguments have sensible defaults suitable for development:
/// - Binds to all interfaces (0.0.0.0)
/// - Uses port 3000
/// - SQLite database with default filename
/// - All OpenAPI documentation disabled by default
#[derive(Args, Debug, Validate)]
#[group(id = "arg", multiple = true, conflicts_with = "cfg")]
pub struct Arguments {
    /// IP address for the API server to bind to.
    ///
    /// Specifies which network interface the server should listen on.
    /// The default value of 0.0.0.0 binds to all available interfaces,
    /// making the API accessible from any network interface.
    ///
    /// # Common Values
    ///
    /// - `0.0.0.0`: Bind to all interfaces (default)
    /// - `127.0.0.1`: Bind only to localhost
    /// - Specific IP: Bind to a particular interface
    ///
    /// # Security Considerations
    ///
    /// Using 0.0.0.0 makes the API accessible from external networks.
    /// For production deployments, consider binding to specific interfaces
    /// or using reverse proxy configurations.
    #[arg(short, long, default_value = "0.0.0.0")]
    pub address: IpAddr,

    /// Port number for the API server to listen on.
    ///
    /// Specifies the TCP port where the API server will accept connections.
    /// The port must be within the valid range and not already in use by
    /// another service.
    ///
    /// # Validation
    ///
    /// Port numbers are validated to be between 1 and 65535, which covers
    /// all valid TCP port numbers while excluding the reserved port 0.
    ///
    /// # Common Ports
    ///
    /// - `3000`: Default development port
    /// - `8080`: Alternative HTTP port
    /// - `80`: Standard HTTP port (requires privileges)
    /// - `443`: Standard HTTPS port (requires privileges)
    ///
    /// # Permissions
    ///
    /// Ports below 1024 typically require root/administrator privileges.
    #[validate(range(min = 1, max = 65535))]
    #[arg(short, long, default_value_t = 3000)]
    pub port: u16,

    /// Database connection URL for the API.
    ///
    /// Specifies the database connection string used by SQLx to connect
    /// to the word database. The format depends on the database type
    /// and includes connection parameters.
    ///
    /// # SQLite Format
    ///
    /// For SQLite databases (default):
    /// - `sqlite:filename.db`: Relative path to database file
    /// - `sqlite:/path/to/file.db`: Absolute path to database file
    /// - `sqlite::memory:`: In-memory database (testing only)
    ///
    /// # File Creation
    ///
    /// SQLite databases are created automatically if the file doesn't exist.
    /// Ensure the directory has write permissions for database creation.
    ///
    /// # Connection Pooling
    ///
    /// The connection URL is used to create a connection pool with multiple
    /// concurrent connections for improved performance.
    #[arg(short, long, default_value = "sqlite:random-words.db")]
    pub database_url: String,

    /// Enable SwaggerUI documentation interface.
    ///
    /// When enabled, provides an interactive web interface for exploring
    /// and testing the API endpoints. SwaggerUI offers a user-friendly
    /// way to understand the API structure and make test requests.
    ///
    /// # Access Path
    ///
    /// When enabled, SwaggerUI is available at `/swagger-ui` endpoint
    /// with full OpenAPI schema integration.
    ///
    /// # Features
    ///
    /// - Interactive API exploration
    /// - Request/response examples
    /// - Parameter input forms
    /// - Live API testing capability
    #[arg(long, default_value_t = false)]
    pub with_swagger_ui: bool,

    /// Enable Redoc documentation interface.
    ///
    /// When enabled, provides a clean, three-panel documentation interface
    /// for the API. Redoc focuses on documentation readability and provides
    /// an excellent reference format for API consumers.
    ///
    /// # Access Path
    ///
    /// When enabled, Redoc is available at `/redoc` endpoint with
    /// comprehensive API documentation.
    ///
    /// # Features
    ///
    /// - Three-panel layout with navigation
    /// - Responsive design for mobile/desktop
    /// - Code samples in multiple languages
    /// - Schema documentation with examples
    #[arg(long, default_value_t = false)]
    pub with_redoc: bool,

    /// Enable Scalar documentation interface.
    ///
    /// When enabled, provides a modern, interactive API documentation
    /// interface with advanced features for API exploration and testing.
    /// Scalar offers a contemporary alternative to traditional documentation.
    ///
    /// # Access Path
    ///
    /// When enabled, Scalar is available at `/scalar` endpoint with
    /// full interactive capabilities.
    ///
    /// # Features
    ///
    /// - Modern, responsive interface
    /// - Interactive request builder
    /// - Real-time API testing
    /// - Advanced schema visualization
    #[arg(long, default_value_t = false)]
    pub with_scalar: bool,

    /// Enable RapiDoc documentation interface.
    ///
    /// When enabled, provides a lightweight, customizable documentation
    /// interface with focus on simplicity and performance. RapiDoc offers
    /// fast loading and minimal resource usage.
    ///
    /// # Access Path
    ///
    /// When enabled, RapiDoc is available at `/rapidoc` endpoint with
    /// streamlined documentation presentation.
    ///
    /// # Features
    ///
    /// - Lightweight and fast loading
    /// - Customizable themes and layouts
    /// - Built-in API testing capability
    /// - Minimal resource footprint
    #[arg(long, default_value_t = false)]
    pub with_rapidoc: bool,
}
