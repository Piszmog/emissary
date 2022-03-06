use std::fmt::{Display, Formatter};
use std::fs;

use serde::Deserialize;
use toml::Value;

/// The configuration for the application.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub http: Http,
    pub proxy: Http,
    pub logging: Logging,
}

/// The HTTP configuration.
#[derive(Debug, Deserialize)]
pub struct Http {
    pub address: String,
    pub port: u16,
}

/// The logging configuration.
#[derive(Debug, Deserialize)]
pub struct Logging {
    pub mode: LoggingMode,
    pub json: JsonLogging,
    pub plain: PlainLogging,
}

/// The logging mode.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoggingMode {
    /// Log in the plain text format.
    Plain,
    /// Log in the JSON format.
    Json,
}

/// The JSON logging configuration.
#[derive(Debug, Deserialize)]
pub struct JsonLogging {
    pub format: Value,
}

/// The plain logging configuration.
#[derive(Debug, Deserialize)]
pub struct PlainLogging {
    pub format: String,
}

/// Load the configuration from the given path.
pub fn read_toml_file(file_path: String) -> Result<Config, Error> {
    fs::read_to_string(file_path)
        .map_err(|err| {
            Error::Io(err)
        })
        .and_then(|contents| {
            toml::from_str(&contents)
                .map_err(|err| {
                    Error::Toml(err)
                })
        })
}

/// The error when loading the configuration.
#[derive(Debug)]
pub enum Error {
    /// The file could not be read.
    Io(std::io::Error),
    /// The file could not be parsed.
    Toml(toml::de::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(err) => write!(f, "{}", err),
            Error::Toml(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {}
