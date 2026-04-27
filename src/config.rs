use self::{metrics::MetricsConfig, router::RouterConfig};
use config::{ConfigError, Environment, File, FileFormat};
use serde::Deserialize;
use std::net::SocketAddr;
use thiserror::Error;
use std::{fs, path::Path};

pub(super) mod metrics;
pub(super) mod router;

#[derive(Error, Debug)]
#[error("{0}. Have you created a Config.toml?")]
pub struct ServerConfigError(#[from] ConfigError);

#[derive(Deserialize, Debug)]
/// Defines the structure of a config file. Extension can be
pub struct ServerConfig {
    /// listening address
    pub listen: SocketAddr,

    // pub routing: Option<RouterConfig>,
    /// routing configuration
    /// required because no other method is currently supported
    pub routing: RouterConfig,

    pub metrics: Option<MetricsConfig>,
}

impl ServerConfig {
    /// reads configuration from Config.toml
    /// (more file exts can be supported through config's features)
    pub fn read() -> Result<Self, ServerConfigError> {
        let path = Path::new("Config.toml");
        if !path.exists() {
            // Write default config
            let default_config = r#"
listen = "0.0.0.0:25565"

[routing]
default = { ip = "127.0.0.1:25565" }
"#;
            fs::write(path, default_config).expect("Failed to write default Config.toml");
        }

        config::Config::builder()
            .add_source(
                Environment::default()
                    .prefix("HOPPER")
                    .separator("_")
                    .try_parsing(true),
            )
            .add_source(File::new("Config.toml", FileFormat::Toml).required(false))
            .build()?
            .try_deserialize()
            .map_err(Into::into)
    }
}
