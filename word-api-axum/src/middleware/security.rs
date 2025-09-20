//! Security headers middleware implementation
//!
//! Provides essential HTTP security headers to protect against common attacks.
//! Headers are applied globally when security_headers_enabled is true in config.

use axum::{body::Body, http::Request, middleware::Next, response::Response};

/// Security headers middleware function
///
/// Adds essential security headers to all responses:
/// - X-Content-Type-Options: nosniff
/// - Referrer-Policy: strict-origin-when-cross-origin
///
/// This function-based middleware is simple and efficient, following
/// the pattern recommended in the Axum documentation.
///
/// Refer to OWASP for more: <https://cheatsheetseries.owasp.org/cheatsheets/HTTP_Headers_Cheat_Sheet.html>
pub async fn security_headers(request: Request<Body>, next: Next) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Prevent MIME type sniffing
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());

    // Specify Content-Type to mitigate XSS vulnerabilities
    headers.insert("content-type", "application/json".parse().unwrap());

    // Control referrer information sent when following links
    headers.insert(
        "referrer-policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );

    // I may not need this since I already use Axum CORS.
    // // Allow API access to specified domains
    // headers.insert(
    //     "access-control-allow-origin",
    //     "https://speak-and-spell.netlify.app/".parse().unwrap(),
    // );

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
        assert_eq!(
            headers.get("referrer-policy").unwrap(),
            "strict-origin-when-cross-origin"
        );
        assert_eq!(headers.get("content-type").unwrap(), "application/json");
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

        // Existing headers should be preserved
        assert_eq!(headers.get("custom-header").unwrap(), "custom-value");
        assert_eq!(headers.get("content-type").unwrap(), "application/json");
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
    }
}
