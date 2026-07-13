use config::{Config, ConfigError};
use std::net::IpAddr;
use std::path::PathBuf;
use url::Url;

#[derive(Clone, serde::Deserialize)]
pub(crate) struct ServiceSettings {
    pub(crate) address: AddressSettings,
    pub(crate) database: DatabaseSettings,
    pub(crate) api: ApiSettings,
}

#[derive(Clone, serde::Deserialize)]
pub(crate) struct AddressSettings {
    pub(crate) ip: IpAddr,
    pub(crate) port: u16,
}

#[derive(Clone, serde::Deserialize)]
pub(crate) struct DatabaseSettings {
    pub(crate) url: Url,
    pub(crate) migrations: Option<PathBuf>,
}

#[derive(Clone, serde::Deserialize)]
pub(crate) struct ApiSettings {
    pub(crate) host_url: Url,
}

impl ServiceSettings {
    pub(crate) fn new() -> Result<Self, ConfigError> {
        let run_mode = std::env::var("LL_RUN_MODE").unwrap_or_else(|_| "dev".into());
        let config_dir = std::env::var("LL_CONFIG_DIR").unwrap_or_else(|_| "./config".into());

        Config::builder()
            .add_source(config::File::with_name(&format!(
                "{config_dir}/settings.json"
            )))
            .add_source(
                config::File::with_name(&format!("{config_dir}/settings.{run_mode}.json"))
                    .required(false),
            )
            .add_source(config::Environment::with_prefix("LL"))
            .build()?
            .try_deserialize()
    }
}
