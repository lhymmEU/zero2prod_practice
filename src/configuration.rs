use secrecy::{Secret, ExposeSecret};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgress://{}:{}@{}:{}/{}",
            self.username, self.password.expose_secret(), self.host, self.port, self.database_name
        ))
    }

    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password.expose_secret(), self.host, self.port
        ))
    }
}

// get a settings struct populated using config files
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let builder = config::Config::builder()
        .set_default("default", "1")?
        .add_source(config::File::new("configuration", config::FileFormat::Yaml))
        .set_override("override", "1")?;
    match builder.build() {
        Ok(config) => {
            config.try_deserialize()
        },
        Err(e) => {
            Err(e)
        }
    }
}