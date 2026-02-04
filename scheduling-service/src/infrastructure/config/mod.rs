use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub data_service: DataServiceSettings,
    pub scheduling_rules: SchedulingRulesSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DataServiceSettings {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SchedulingRulesSettings {
    pub min_day_off_per_week: u8,
    pub max_day_off_per_week: u8,
    pub no_morning_after_evening: bool,
    pub max_daily_shift_diff: u32,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let environment = std::env::var("RUN_ENV").unwrap_or_else(|_| "development".to_string());

        let config = Config::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name(&format!("config/{}", environment)).required(false))
            .add_source(config::Environment::with_prefix("APP").separator("__"))
            .build()?;

        config.try_deserialize()
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}
