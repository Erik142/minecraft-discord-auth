use config::Environment;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub pg: deadpool_postgres::Config,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .add_source(Environment::default().separator("__"))
            .build()?
            .try_deserialize()
    }
}
