//! Random Word API Library
//!
//! This library provides the core functionality for the random word API,
//! including models, handlers, routing, and configuration management.
//! It is designed to be used both as a standalone binary and as a library
//! for testing and integration purposes.

/// CLI argument parsing and configuration
pub mod cli;

/// Error handling types and conversions
pub mod error;

/// HTTP request handlers
pub mod handlers;

/// Data models and business logic
pub mod models;

/// Route configuration and middleware
pub mod routes;

/// Application state management
pub mod state;
