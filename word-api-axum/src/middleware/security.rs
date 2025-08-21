//! Security headers middleware implementation
//!
//! Provides essential HTTP security headers to protect against common attacks.
//! Headers are applied globally when security_headers_enabled is true in config.

use axum::{body::Body, http::Request, middleware::Next, response::Response};

/// Security headers middleware function
///
/// Adds essential security headers to all responses:
/// - X-Content-Type-Options: nosniff
/// - X-Frame-Options: DENY
/// - X-XSS-Protection: 1; mode=block
/// - Referrer-Policy: strict-origin-when-cross-origin
///
/// This function-based middleware is simple and efficient, following
/// the pattern recommended in the Axum documentation.
pub async fn security_headers(request: Request<Body>, next: Next) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Prevent MIME type sniffing
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());

    // Prevent page from being displayed in a frame/iframe
    headers.insert("x-frame-options", "DENY".parse().unwrap());

    // Enable XSS filtering in older browsers
    headers.insert("x-xss-protection", "1; mode=block".parse().unwrap());

    // Control referrer information sent when following links
    headers.insert(
        "referrer-policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use axum_test::TestServer;

    async fn test_handler() -> &'static str {
        "Hello, world!"
    }

    #[tokio::test]
    async fn test_security_headers_applied() {
        let app = Router::new()
            .route("/", get(test_handler))
            .layer(axum::middleware::from_fn(security_headers));

        let server = TestServer::new(app).unwrap();
        let response = server.get("/").await;

        let headers = response.headers();

        // Check that all security headers are present with correct values
        assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
        assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
        assert_eq!(headers.get("x-xss-protection").unwrap(), "1; mode=block");
        assert_eq!(
            headers.get("referrer-policy").unwrap(),
            "strict-origin-when-cross-origin"
        );
    }

    #[tokio::test]
    async fn test_security_headers_preserve_existing() {
        async fn handler_with_custom_header() -> Response {
            let mut response = Response::new("Hello with custom header".into());
            response
                .headers_mut()
                .insert("custom-header", "custom-value".parse().unwrap());
            response
        }

        let app = Router::new()
            .route("/", get(handler_with_custom_header))
            .layer(axum::middleware::from_fn(security_headers));

        let server = TestServer::new(app).unwrap();
        let response = server.get("/").await;

        let headers = response.headers();

        // Security headers should be added
        assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
        assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");

        // Existing headers should be preserved
        assert_eq!(headers.get("custom-header").unwrap(), "custom-value");
    }

    #[tokio::test]
    async fn test_security_headers_with_error_response() {
        async fn error_handler() -> Result<&'static str, axum::http::StatusCode> {
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }

        let app = Router::new()
            .route("/error", get(error_handler))
            .layer(axum::middleware::from_fn(security_headers));

        let server = TestServer::new(app).unwrap();
        let response = server.get("/error").await;

        // Security headers should be applied even to error responses
        let headers = response.headers();
        assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
        assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
    }
}
