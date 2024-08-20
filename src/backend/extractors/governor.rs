//! # Cookie-based Rate Limiting Module
//!
//! This module implements a custom key extractor for tower_governor,
//! allowing rate limiting based on a session ID cookie.
//!
//! It provides:
//! - A `CookieExtractor` struct that implements `KeyExtractor` trait
//! - Functionality to extract rate limiting keys from the 'sid' cookie in requests and fall back to the client's IP address
use axum::http::Request;
use axum::http::{header::FORWARDED, HeaderMap};
use axum_extra::extract::CookieJar;
use forwarded_header_value::{ForwardedHeaderValue, Identifier};
use std::net::IpAddr;
use tower_governor::{
    key_extractor::{KeyExtractor, PeerIpKeyExtractor},
    GovernorError,
};
const X_REAL_IP: &str = "x-real-ip";
const X_FORWARDED_FOR: &str = "x-forwarded-for";
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
                let headers = req.headers();
                maybe_x_forwarded_for(headers)
                    .or_else(|| maybe_x_real_ip(headers))
                    .or_else(|| maybe_forwarded(headers))
                    .or_else(|| {
                        dioxus_logger::tracing::warn!(
                            "Unable to extract key from cookie or forwarded headers, falling back to peer IP address"
                        );
                        PeerIpKeyExtractor.extract(req).ok()
                    })
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
/// Tries to parse the `x-forwarded-for` header
fn maybe_x_forwarded_for(headers: &HeaderMap) -> Option<IpAddr> {
    headers
        .get(X_FORWARDED_FOR)
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| s.split(',').find_map(|s| s.trim().parse::<IpAddr>().ok()))
}
/// Tries to parse the `x-real-ip` header
fn maybe_x_real_ip(headers: &HeaderMap) -> Option<IpAddr> {
    headers
        .get(X_REAL_IP)
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| s.parse::<IpAddr>().ok())
}
/// Tries to parse `forwarded` headers
fn maybe_forwarded(headers: &HeaderMap) -> Option<IpAddr> {
    headers.get_all(FORWARDED).iter().find_map(|hv| {
        hv.to_str()
            .ok()
            .and_then(|s| ForwardedHeaderValue::from_forwarded(s).ok())
            .and_then(|f| {
                f.iter()
                    .filter_map(|fs| fs.forwarded_for.as_ref())
                    .find_map(|ff| match ff {
                        Identifier::SocketAddr(a) => Some(a.ip()),
                        Identifier::IpAddr(ip) => Some(*ip),
                        _ => None,
                    })
            })
    })
}
