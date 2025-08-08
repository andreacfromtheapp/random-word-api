//! OpenAPI documentation routes module
//!
//! This module provides comprehensive OpenAPI documentation interface configuration
//! for the random word API. It supports multiple documentation formats including
//! SwaggerUI, Redoc, Scalar, and RapiDoc, each offering different features and
//! user experiences for API exploration and testing.
//!
//! # Documentation Interfaces
//!
//! The module supports four popular OpenAPI documentation interfaces:
//!
//! - `SwaggerUI`: Interactive API explorer with built-in testing capabilities
//! - `Redoc`: Clean, three-panel documentation layout with excellent readability
//! - `Scalar`: Modern interface with advanced features and contemporary design
//! - `RapiDoc`: Lightweight, fast-loading documentation with minimal resource usage
//!
//! # Configuration-Driven Routing
//!
//! Documentation routes are dynamically enabled based on application configuration:
//! - Each interface can be independently enabled or disabled
//! - Configuration comes from CLI arguments, environment variables, or config files
//! - Disabled interfaces are not included in the router for optimal performance
//!
//! # OpenAPI Schema Generation
//!
//! The module uses utoipa for automatic OpenAPI schema generation from:
//! - Handler function annotations and documentation
//! - Model struct schemas with validation rules
//! - Comprehensive tag organization for endpoint grouping
//! - Response schema definitions with status codes
//!
//! # Performance Considerations
//!
//! Documentation interfaces are optional to minimize resource usage:
//! - Production deployments can disable all interfaces
//! - Development environments can enable specific interfaces
//! - Each interface adds to memory usage and startup time
//!
//! # CORS Configuration
//!
//! Documentation routes include permissive CORS settings to support:
//! - Cross-origin access from development tools
//! - Integration with external API testing platforms
//! - Browser-based documentation access from various origins

// OpenAPI documentation configuration
use axum::Router;
use utoipa::OpenApi;

use crate::handlers::{admin::*, healthcheck::*, word::*};
use crate::models::word::{GetWord, UpsertWord, Word};
use crate::state::AppState;

/// OpenAPI specification structure with comprehensive endpoint documentation.
///
/// This struct defines the complete OpenAPI specification for the random word API
/// using utoipa's derive macro. It includes all endpoints, schemas, and metadata
/// necessary for generating comprehensive API documentation across all supported
/// interfaces.
///
/// # Path Configuration
///
/// Manually configured paths ensure all endpoints are properly documented:
/// - Health check endpoints for monitoring integration
/// - Public word retrieval endpoints for end-user consumption
/// - Administrative word management endpoints for content management
///
/// # Schema Components
///
/// Includes all data transfer objects and models:
/// - Word: Complete word structure with metadata
/// - GetWord: Public word structure without metadata
/// - UpsertWord: Word creation and update structure with validation
///
/// # Tag Organization
///
/// Endpoints are organized into logical groups:
/// - healthcheck_endpoints: System monitoring and status
/// - public_endpoints: Publicly accessible word retrieval
/// - administration_endpoints: Protected word management operations
///
/// # Documentation Generation
///
/// This structure is used by all documentation interfaces to generate:
/// - Interactive API explorers with testing capabilities
/// - Static documentation with comprehensive endpoint details
/// - Schema definitions with validation rules and examples
#[derive(OpenApi)]
#[openapi(
    paths(
        // Health check endpoints
        alive,
        ping,
        // Public word endpoints
        word_random,
        // Administrative endpoints
        word_list,
        word_create,
        word_read,
        word_update,
        word_delete,
    ),
    components(
        schemas(Word, GetWord, UpsertWord)
    ),
    tags(
        (name = "healthcheck_endpoints", description = "Health check and system status endpoints"),
        (name = "public_endpoints", description = "Public word retrieval endpoints"),
        (name = "administration_endpoints", description = "Administrative word management endpoints"),
    ),
)]
pub struct ApiDoc;

/// Creates SwaggerUI documentation router with interactive API exploration.
///
/// This function sets up SwaggerUI, an interactive web interface that allows
/// users to explore and test API endpoints directly from the browser. It
/// provides comprehensive API documentation with built-in request testing
/// capabilities.
///
/// # Features
///
/// SwaggerUI provides:
/// - Interactive endpoint exploration with parameter input forms
/// - Live API testing with request/response visualization
/// - Comprehensive schema documentation with examples
/// - Authentication testing capabilities
/// - Request/response code generation
///
/// # Access Path
///
/// The interface is available at `/swagger-ui` with the OpenAPI specification
/// served from `/api-docs/openapi.json` for dynamic schema loading.
///
/// # Use Cases
///
/// Ideal for:
/// - API development and debugging
/// - Client application development
/// - API testing and validation
/// - Documentation review and verification
pub fn create_swagger_routes() -> Router {
    use utoipa_swagger_ui::SwaggerUi;

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

/// Creates Redoc documentation router with clean, readable interface.
///
/// This function sets up Redoc, a documentation interface focused on readability
/// and clean presentation. It provides excellent documentation viewing with a
/// three-panel layout optimized for comprehensive API reference.
///
/// # Features
///
/// Redoc provides:
/// - Three-panel layout with navigation, content, and examples
/// - Responsive design for desktop and mobile viewing
/// - Code samples in multiple programming languages
/// - Comprehensive schema documentation with nested object support
/// - Search functionality for large API specifications
///
/// # Access Path
///
/// The interface is available at `/redoc` with embedded OpenAPI specification
/// for immediate documentation access without external dependencies.
///
/// # Use Cases
///
/// Ideal for:
/// - API reference documentation
/// - Client developer onboarding
/// - Comprehensive API overview
/// - Documentation publishing and sharing
pub fn create_redoc_routes() -> Router {
    use utoipa_redoc::{Redoc, Servable};

    Router::new().merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
}

/// Creates Scalar documentation router with modern, advanced interface.
///
/// This function sets up Scalar, a contemporary documentation interface with
/// advanced features and modern design principles. It provides an excellent
/// user experience with interactive capabilities and visual appeal.
///
/// # Features
///
/// Scalar provides:
/// - Modern, responsive interface with contemporary design
/// - Advanced interactive request builder with form validation
/// - Real-time API testing with response visualization
/// - Schema visualization with relationship mapping
/// - Dark/light theme support for user preference
///
/// # Access Path
///
/// The interface is available at `/scalar` with integrated OpenAPI specification
/// loading for seamless documentation access.
///
/// # Use Cases
///
/// Ideal for:
/// - Modern API documentation presentation
/// - Advanced API testing and development
/// - Client application prototyping
/// - Interactive API exploration
pub fn create_scalar_routes() -> Router {
    use utoipa_scalar::{Scalar, Servable};

    Router::new().merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
}

/// Creates RapiDoc documentation router with lightweight, fast interface.
///
/// This function sets up RapiDoc, a performance-focused documentation interface
/// designed for speed and minimal resource usage. It provides comprehensive
/// documentation capabilities while maintaining excellent performance characteristics.
///
/// # Features
///
/// RapiDoc provides:
/// - Lightweight implementation with fast loading times
/// - Customizable themes and layout options
/// - Built-in API testing with request/response handling
/// - Minimal resource footprint and memory usage
/// - Search functionality for endpoint discovery
///
/// # Access Path
///
/// The interface is available at `/rapidoc` with OpenAPI specification
/// loaded from `/api-docs/openapi.json` for dynamic documentation.
///
/// # Use Cases
///
/// Ideal for:
/// - Performance-sensitive documentation needs
/// - Resource-constrained environments
/// - Fast documentation loading and browsing
/// - Minimal overhead API reference
pub fn create_rapidoc_routes() -> Router {
    use utoipa_rapidoc::RapiDoc;

    Router::new().merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
}

/// Creates the complete OpenAPI documentation router with configurable interfaces.
///
/// This function assembles all available documentation interfaces into a unified
/// router based on the application configuration. It dynamically enables only
/// the documentation interfaces that are configured to be active, optimizing
/// resource usage and startup performance.
///
/// # Dynamic Configuration
///
/// The function checks the application configuration to determine which
/// documentation interfaces to enable:
/// - Reads configuration from the shared application state
/// - Only includes enabled interfaces in the final router
/// - Skips disabled interfaces to reduce memory usage
///
/// # Router Assembly
///
/// The function conditionally merges documentation routers:
/// - SwaggerUI router for interactive API exploration
/// - Redoc router for clean documentation presentation
/// - Scalar router for modern documentation experience
/// - RapiDoc router for lightweight documentation access
///
/// # CORS Configuration
///
/// Applies permissive CORS settings to support:
/// - Cross-origin access from development tools
/// - Browser-based documentation viewing
/// - Integration with external API testing platforms
///
/// # Arguments
///
/// * `state` - Shared application state containing configuration and database pool
/// * `origins` - List of allowed CORS origins for cross-origin documentation access
///
/// # Returns
///
/// A configured Axum Router containing only the enabled documentation interfaces
/// with appropriate CORS policies and middleware for optimal documentation access.
///
/// # Performance Impact
///
/// Only enabled interfaces are included in the router, ensuring minimal
/// performance impact when documentation is disabled for production deployments.
pub fn create_apidocs_routes(state: AppState, origins: Vec<http::HeaderValue>) -> Router {
    use http::Method;
    use tower_http::cors::CorsLayer;

    let mut router = Router::new();

    // Get the config to check which documentation routes to enable
    if let Ok(config) = state.config.lock() {
        if config.openapi.enable_swagger_ui {
            router = router.merge(create_swagger_routes());
        }
        if config.openapi.enable_redoc {
            router = router.merge(create_redoc_routes());
        }
        if config.openapi.enable_scalar {
            router = router.merge(create_scalar_routes());
        }
        if config.openapi.enable_rapidoc {
            router = router.merge(create_rapidoc_routes());
        }
    }

    router.layer(
        CorsLayer::new()
            .allow_methods([Method::POST, Method::GET, Method::PUT, Method::DELETE])
            .allow_origin(origins.clone()),
    )
}
