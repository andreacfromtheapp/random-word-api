//! HTTP request handlers
//!
//! # Modules
//! - `admin`: Word management endpoints (requires auth)
//! - `auth`: Authentication endpoints for login
//! - `healthcheck`: System status endpoints
//! - `word`: Public word retrieval endpoints
//!
//! All handlers return JSON responses and use centralized error handling.

pub mod admin;
pub mod auth;
pub mod healthcheck;
pub mod word;
