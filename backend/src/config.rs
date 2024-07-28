use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct RateLimiting {
    pub requests_per_second: u64,
    pub burst_size: u32,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct OAuth {
    pub oauth_redirect_uri: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct AppConfig {
    pub oauth: OAuth,
    pub ratelimiting: RateLimiting,
}

impl AppConfig {
    pub fn new<S: AsRef<str>>(base: S) -> Result<Self, ConfigError> {
        let run_mode = if cfg!(debug_assertions) {
            "development"
        } else {
            "production"
        };

        let base = base.as_ref();

        let s = Config::builder()
            .add_source(File::with_name(&format!("{base}/config/default")).required(true))
            .add_source(File::with_name(&format!("{base}/config/{}", run_mode)).required(false))
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
        let config = AppConfig::new("../").unwrap();
        assert_eq!(
            config.oauth.oauth_redirect_uri,
            "http://localhost:8000/v1/auth/github/callback"
        );
        assert_eq!(config.ratelimiting.requests_per_second, 5);
        assert_eq!(config.ratelimiting.burst_size, 10);
    }
}
