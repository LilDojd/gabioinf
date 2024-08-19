//! # Cookie-based Rate Limiting Module
//!
//! This module implements a custom key extractor for tower_governor,
//! allowing rate limiting based on a session ID cookie.
//!
//! It provides:
//! - A `CookieExtractor` struct that implements `KeyExtractor` trait
//! - Functionality to extract rate limiting keys from the 'sid' cookie in requests and fall back to the client's IP address
use axum::http::Request;
use axum_extra::extract::CookieJar;
use tower_governor::{
    key_extractor::{KeyExtractor, PeerIpKeyExtractor, SmartIpKeyExtractor},
    GovernorError,
};
/// A key extractor that uses the 'sid' cookie for rate limiting
#[derive(Clone)]
pub struct CookieExtractor;
impl KeyExtractor for CookieExtractor {
    /// The type of the key used for rate limiting
    type Key = String;
    /// Extracts the rate limiting key from the request
    ///
    /// # Arguments
    ///
    /// * `req` - The incoming HTTP request
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The value of the 'id' cookie if present
    /// * `Err(GovernorError::UnableToExtractKey)` - If the 'id' cookie is not found
    fn extract<B>(&self, req: &Request<B>) -> Result<Self::Key, GovernorError> {
        let jar = CookieJar::from_headers(req.headers());
        jar.get("id")
            .map(|cookie| cookie.value().to_string())
            .or_else(|| {
                SmartIpKeyExtractor
                    .extract(req)
                    .ok()
                    .map(|ip| ip.to_string())
            })
            .or_else(|| {
                dioxus_logger::tracing::warn!("Unable to extract key from cookie or forwarded headers, falling back to peer IP address");
                PeerIpKeyExtractor
                    .extract(req)
                    .ok()
                    .map(|ip| ip.to_string())
            })
            .ok_or(GovernorError::UnableToExtractKey)
    }
    /// Returns the name of this key extractor
    ///
    /// # Returns
    ///
    /// A static string slice containing the name "CookieExtractor"
    fn name(&self) -> &'static str {
        "CookieExtractor"
    }
}
