use std::net::TcpListener;

use config::{Config, ConfigError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerConfigError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
}

#[derive(Debug)]
pub struct ServerConfig {
    log_level: String,
    pub(crate) tcp_listener: TcpListener,
}

impl ServerConfig {
    #[tracing::instrument()]
    pub fn load(config: &mut Config) -> Result<Self, ServerConfigError> {
        let settings = config;
        settings
            .merge(config::File::with_name("letmein.toml"))?
            .merge(config::Environment::with_prefix("LETMEIN_SERVER"))?;

        let env: String = settings.get("env")?;

        let port: u16 = match env.as_str() {
            "dev" => 0,
            _ => settings.get("port").unwrap_or(0),
        };

        let host: &str = match env.as_str() {
            "dev" => settings.get("host").unwrap_or("127.0.0.1"),
            _ => "0.0.0.0",
        };

        Ok(Self {
            log_level: settings.get("log_level")?,
            tcp_listener: TcpListener::bind(format!("{}:{}", host, port))?,
        })
    }

    pub fn host(&self) -> String {
        self.tcp_listener.local_addr().unwrap().ip().to_string()
    }

    pub fn log_level(&self) -> &str {
        self.log_level.as_str()
    }

    pub fn port(&self) -> u16 {
        self.tcp_listener.local_addr().unwrap().port()
    }
}
