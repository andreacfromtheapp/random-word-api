//! Models module
//!
//! This module contains all data models and business logic structures for the
//! random word API. It provides the core data types, validation logic, and
//! database interaction methods that form the foundation of the application.
//!
//! # Module Organization
//!
//! The models are organized into two primary categories:
//!
//! - `apiconfig`: Configuration management and application settings
//! - `word`: Word data structures, validation, and database operations
//!
//! # Architecture
//!
//! All models follow consistent patterns for:
//! - Data validation using the `validator` crate
//! - Database interaction through SQLx with connection pooling
//! - Serialization support for JSON APIs using Serde
//! - OpenAPI schema generation using utoipa
//! - Error handling with comprehensive error types
//!
//! # Validation Strategy
//!
//! Models implement multi-layered validation:
//! - Field-level validation using derive macros
//! - Custom validation functions for domain-specific rules
//! - Database constraint validation through unique indexes
//! - Business logic validation in accessor methods
//!
//! # Database Integration
//!
//! All models are designed to work seamlessly with SQLite through SQLx:
//! - Automatic row mapping using derive macros
//! - Prepared statement support for security
//! - Connection pool integration for performance
//! - Transaction support for data consistency
//!
//! # API Integration
//!
//! Models support comprehensive API integration:
//! - JSON serialization for HTTP responses
//! - OpenAPI schema generation for documentation
//! - Request deserialization with validation
//! - Error conversion to HTTP status codes

pub mod apiconfig;
pub mod word;
