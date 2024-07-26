use config::{Config, ConfigError, File};
use serde::Deserialize;

/// Application configuration
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub oauth_redirect_uri: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = if cfg!(debug_assertions) {
            "development"
        } else {
            "production"
        };

        let s = Config::builder()
            .add_source(File::with_name("config/default").required(true))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(true))
            .build()?;

        s.try_deserialize()
    }
}
