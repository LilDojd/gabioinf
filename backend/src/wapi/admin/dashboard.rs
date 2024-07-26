//! Admin dashboard handler.
//!
//! This module contains the handler function for serving the admin dashboard HTML page.

use axum::{
    http::{header, StatusCode},
    response::{Html, IntoResponse},
};

/// Handler for serving the admin dashboard.
///
/// This function returns the HTML content for the admin dashboard.
/// The HTML content is embedded in the binary at compile time using `include_str!`.
///
/// # Returns
///
/// Returns an `impl IntoResponse` which resolves to:
/// - A `200 OK` status code
/// - A `Content-Type` header set to `text/html; charset=utf-8`
/// - The HTML content of the admin dashboard
///
/// # Security Considerations
///
/// - This endpoint should be protected and only accessible to authenticated administrators.
/// - Ensure that the HTML content doesn't contain any sensitive information, as it's embedded in the binary.
/// - Consider implementing CSP (Content Security Policy) headers for enhanced security.
///
/// # Performance Note
///
/// The HTML content is included at compile time, which means:
/// - Fast response times as the content is in memory
/// - No file I/O is needed to serve the dashboard
/// - Any changes to the HTML require recompiling the application
///
/// # Example
///
/// ```json
/// GET /admin
/// Authorization: Bearer <admin-token>
/// ```
///
/// Successful response:
/// ```json
/// HTTP/1.1 200 OK
/// Content-Type: text/html; charset=utf-8
///
/// <!DOCTYPE html>
/// <html lang="en">
/// ...
/// </html>
/// ```
pub async fn admin_dashboard() -> impl IntoResponse {
    let html = include_str!("./templates/admin_dashboard.html");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        Html(html),
    )
}
