#![allow(unused)]
use crate::hide::Hide;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
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
pub struct AppConfig {
    pub domain: String,
    pub ratelimiting: RateLimiting,
    pub database: DatabaseConfig,
    pub gabioinf: GabioinfConfig,
}
impl AppConfig {
    pub fn new<S: AsRef<str>>(base: S) -> Result<Self, ConfigError> {
        let run_mode = if cfg!(debug_assertions) { "development" } else { "production" };
        let base = base.as_ref();
        let s = Config::builder()
            .add_source(
                File::with_name(&format!("{base}/config/default")).required(true),
            )
            .add_source(
                File::with_name(&format!("{base}/config/{}", run_mode)).required(false),
            )
            .add_source(
                Environment::with_prefix("DATABASE")
                    .keep_prefix(true)
                    .separator("_")
                    .convert_case(config::Case::UpperSnake),
            )
            .add_source(
                Environment::with_prefix("GABIOINF")
                    .keep_prefix(true)
                    .separator("_")
                    .convert_case(config::Case::UpperSnake),
            )
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
        let config = AppConfig::new("./").unwrap();
        assert_eq!(config.ratelimiting.requests_per_second, 5);
        assert_eq!(config.ratelimiting.burst_size, 10);
    }
}
