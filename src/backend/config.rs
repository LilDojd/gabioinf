use crate::hide::Hide;
use axum_extra::extract::cookie::Key;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Deserializer, de};
#[derive(Debug, Deserialize)]
pub struct RateLimiting {
    pub requests_per_second: u64,
    pub burst_size: u32,
}
#[derive(Debug, Deserialize)]
pub struct GabioinfConfig {
    pub id: String,
    pub secret: Hide<String>,
}
#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: Hide<String>,
}
#[derive(Debug, Deserialize)]
pub struct SessionConfig {
    #[serde(deserialize_with = "deserialize_session_key")]
    pub secret: Key,
}
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub domain: String,
    pub ratelimiting: RateLimiting,
    pub database: DatabaseConfig,
    pub gabioinf: GabioinfConfig,
    pub session: SessionConfig,
}

fn deserialize_session_key<'de, D>(deserializer: D) -> Result<Key, D::Error>
where
    D: Deserializer<'de>,
{
    let encoded = String::deserialize(deserializer)?;
    decode_session_key(&encoded).map_err(de::Error::custom)
}

fn decode_session_key(encoded: &str) -> Result<Key, &'static str> {
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| "SESSION_SECRET must be valid standard base64")?;
    Key::try_from(bytes.as_slice()).map_err(|_| "SESSION_SECRET must decode to at least 64 bytes")
}
impl AppConfig {
    pub fn new<S: AsRef<str>>(base: S) -> Result<Self, ConfigError> {
        let run_mode = if cfg!(debug_assertions) {
            "development"
        } else {
            "production"
        };
        let domain = std::env::var("DOMAIN_URL").ok();
        let base = base.as_ref();
        let s = Config::builder()
            .add_source(File::with_name(&format!("{base}/config/default")).required(true))
            .add_source(File::with_name(&format!("{base}/config/{run_mode}")).required(false))
            .add_source(
                Environment::with_prefix("DATABASE")
                    .keep_prefix(true)
                    .separator("_")
                    .convert_case(config::Case::Lower),
            )
            .add_source(
                Environment::with_prefix("GABIOINF")
                    .keep_prefix(true)
                    .separator("_")
                    .convert_case(config::Case::Lower),
            )
            .add_source(
                Environment::with_prefix("SESSION")
                    .keep_prefix(true)
                    .separator("_")
                    .convert_case(config::Case::Lower),
            )
            .set_override_option("domain", domain)?
            .build()?;
        s.try_deserialize()
    }
    pub fn new_local() -> Result<Self, ConfigError> {
        Self::new(".")
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let config = Config::builder()
            .add_source(File::with_name("./config/default").required(true))
            .build()
            .unwrap();
        assert_eq!(
            config.get_int("ratelimiting.requests_per_second").unwrap(),
            5
        );
        assert_eq!(config.get_int("ratelimiting.burst_size").unwrap(), 10);
    }

    #[test]
    fn rejects_malformed_session_secret() {
        assert_eq!(
            decode_session_key("not base64").unwrap_err(),
            "SESSION_SECRET must be valid standard base64"
        );
    }

    #[test]
    fn rejects_short_session_secret() {
        let encoded = STANDARD.encode([0; 63]);
        assert_eq!(
            decode_session_key(&encoded).unwrap_err(),
            "SESSION_SECRET must decode to at least 64 bytes"
        );
    }

    #[test]
    fn accepts_64_byte_session_secret() {
        let material = [42; 64];
        let key = decode_session_key(&STANDARD.encode(material)).unwrap();
        assert_eq!(key.master(), material);
    }
}
