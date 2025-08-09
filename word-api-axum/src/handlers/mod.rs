//! HTTP handlers module
//!
//! This module contains all HTTP request handlers for the random word API,
//! organized by functionality and access level. The handlers are responsible
//! for processing incoming HTTP requests, validating input, calling business
//! logic from the model layer, and returning appropriate HTTP responses.
//!
//! # Module Organization
//!
//! The handlers are split into three main categories:
//!
//! - `admin`: Administrative endpoints requiring authentication and admin privileges
//! - `healthcheck`: Public utility endpoints for health checks and API status
//! - `word`: Public endpoints for word retrieval functionality
//!
//! # Architecture
//!
//! All handlers follow a consistent pattern:
//! - Extract data from HTTP requests (path parameters, JSON bodies, etc.)
//! - Validate input data using the model layer validation
//! - Call appropriate model methods to perform business logic
//! - Convert results to JSON responses with proper HTTP status codes
//! - Handle errors gracefully using the centralized error handling system
//!
//! # Error Handling
//!
//! All handlers return `Result<T, AppError>` where `AppError` is automatically
//! converted to appropriate HTTP responses by Axum middleware. This ensures
//! consistent error formatting and proper HTTP status codes across all endpoints.
//!
//! # OpenAPI Documentation
//!
//! Each handler function includes comprehensive `#[utoipa::path]` attributes
//! that generate OpenAPI documentation, including request/response schemas,
//! status codes, and detailed descriptions for API consumers.

pub mod admin;
pub mod healthcheck;
pub mod word;
