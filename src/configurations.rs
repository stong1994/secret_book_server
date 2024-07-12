use anyhow::{Context, Result};
use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Clone, serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

impl TryFrom<Config> for Settings {
    type Error = config::ConfigError;
    fn try_from(config: Config) -> Result<Self, Self::Error> {
        let host: String = config.get("application.host")?;
        let port: u16 = config.get("application.port")?;
        let base_url: String = config.get("application.base_url")?;
        let db_url: String = config.get("database.url")?;

        Ok(Settings {
            database: DatabaseSettings { url: db_url },
            application: ApplicationSettings {
                host,
                port,
                base_url,
            },
        })
    }
}

#[derive(Clone, serde::Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
    pub base_url: String,
}

#[derive(Clone, serde::Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
}

pub fn get_configuration() -> Result<Settings> {
    let settings = config::Config::builder()
        .add_source(File::with_name("config.toml"))
        .add_source(Environment::with_prefix("SECRET"))
        .build()
        .context("Failed to build configuration")?;

    Ok(settings.try_into()?)
}
