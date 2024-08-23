#![allow(dead_code)]
use axum::http::{header, HeaderValue};
use axum::response::Response;
use std::time::Duration;

// An opinionated Enum for different cache control states
#[derive(Debug, Clone)]
pub enum CachePolicy {
    NoCache,
    NoStore,
    Private(Duration),
    Immutable,
    Revalidate {
        ttl: Duration,
        stale_ttl: Option<Duration>,
    },
}

impl CachePolicy {
    pub fn to_header_value(&self) -> String {
        match self {
            CachePolicy::NoCache => "no-cache".to_string(),
            CachePolicy::NoStore => "no-store".to_string(),
            CachePolicy::Private(duration) => format!("private, max-age={}", duration.as_secs()),
            CachePolicy::Immutable => "max-age=31536000, immutable".to_string(),
            CachePolicy::Revalidate { ttl, stale_ttl } => {
                let mut value = format!("max-age={}", ttl.as_secs());
                if let Some(stale) = stale_ttl {
                    value.push_str(&format!(", stale-while-revalidate={}", stale.as_secs()));
                }
                value
            }
        }
    }
}

// Trait for responses that can use cache policies
pub trait CacheableResponse {
    fn set_cache_policy(&mut self, policy: CachePolicy);
    fn get_etag(&self) -> Option<String>;
    fn set_etag(&mut self, etag: Option<String>);
}

impl CacheableResponse for Response {
    fn set_cache_policy(&mut self, policy: CachePolicy) {
        self.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_str(&policy.to_header_value()).unwrap(),
        );
    }

    fn get_etag(&self) -> Option<String> {
        self.headers()
            .get(header::ETAG)
            .and_then(|v| v.to_str().ok())
            .map(String::from)
    }

    fn set_etag(&mut self, etag: Option<String>) {
        if let Some(etag) = etag {
            self.headers_mut()
                .insert(header::ETAG, HeaderValue::from_str(&etag).unwrap());
        }
    }
}

pub(crate) fn generate_etag(uri_path: &str) -> Option<String> {
    if let Ok(metadata) = std::fs::metadata(uri_path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(std::time::SystemTime::UNIX_EPOCH) {
                return Some(format!("\"{:x}-{:x}\"", duration.as_secs(), metadata.len()));
            }
        }
    }
    None
}
