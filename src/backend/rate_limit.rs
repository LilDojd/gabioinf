use axum::http::Request;
use std::net::IpAddr;
use tower_governor::{
    GovernorError,
    key_extractor::{KeyExtractor, PeerIpKeyExtractor},
};

const FLY_CLIENT_IP: &str = "fly-client-ip";

#[derive(Clone)]
pub struct FlyClientIpExtractor;

impl KeyExtractor for FlyClientIpExtractor {
    type Key = IpAddr;

    fn name(&self) -> &'static str {
        "FlyClientIpExtractor"
    }

    fn extract<B>(&self, req: &Request<B>) -> Result<Self::Key, GovernorError> {
        match req
            .headers()
            .get(FLY_CLIENT_IP)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok())
        {
            Some(ip) => Ok(ip),
            None => PeerIpKeyExtractor.extract(req),
        }
    }
}
