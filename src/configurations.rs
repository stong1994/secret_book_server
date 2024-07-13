use anyhow::{Context, Result};
use config::{Config, Environment, File};

#[derive(Clone, serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub log: LogSettings,
}

impl TryFrom<Config> for Settings {
    type Error = config::ConfigError;
    fn try_from(config: Config) -> Result<Self, Self::Error> {
        let host: String = config.get("application.host")?;
        let port: u16 = config.get("application.port")?;
        let db_url: String = config.get("database.url")?;
        let log_dir: String = config.get("log.dir")?;
        let log_level: String = config.get("log.level")?;

        Ok(Settings {
            database: DatabaseSettings { url: db_url },
            application: ApplicationSettings { host, port },
            log: LogSettings {
                dir: log_dir,
                level: log_level,
            },
        })
    }
}

#[derive(Clone, serde::Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(Clone, serde::Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
}

#[derive(Clone, serde::Deserialize)]
pub struct LogSettings {
    pub level: String,
    pub dir: String,
}

pub fn get_configuration() -> Result<Settings> {
    let settings = config::Config::builder()
        .set_default("application.host", "localhost")?
        .set_default("application.port", 12345)?
        .set_default("database.url", ".secret/secret.db")?
        .set_default("log.dir", ".secret/logs/")?
        .set_default("log.level", "info")?
        .add_source(File::with_name("config.toml"))
        .add_source(Environment::with_prefix("SECRET").separator("_"))
        .build()
        .context("Failed to build configuration")?;

    Ok(settings.try_into()?)
}
