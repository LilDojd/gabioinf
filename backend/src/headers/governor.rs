//! # Custom Rate Limiting Module
//!
//! This module implements custom rate limiting functionality using tower_governor.
//! It defines a custom header and key extractor for use with the governor middleware.
//!
//! The module provides:
//! - A custom header `CustomHeader` for rate limiting purposes
//! - A custom key extractor `CustomHeaderExtractor` to extract rate limiting keys from requests
//!
//! This allows for user-based rate limiting using a custom header instead of IP addresses.

use axum::http::Request;
use axum_extra::extract::CookieJar;
use tower_governor::{key_extractor::KeyExtractor, GovernorError};

#[derive(Clone)]
pub struct CookieExtractor;

impl KeyExtractor for CookieExtractor {
    type Key = String;

    fn extract<B>(&self, req: &Request<B>) -> Result<Self::Key, GovernorError> {
        let jar = CookieJar::from_headers(req.headers());
        jar.get("sid")
            .map(|cookie| cookie.value().to_string())
            .ok_or(GovernorError::UnableToExtractKey)
    }

    fn name(&self) -> &'static str {
        "CookieExtractor"
    }
}
