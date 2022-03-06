use std::fmt::{Display, Formatter};
use std::net::ToSocketAddrs;

use serde_json::Value as Json;
use toml::Value as Toml;
use url::Url;

pub fn to_url(addr: String, port: u16) -> Result<Url, UrlError> {
    (addr, port)
        .to_socket_addrs()
        .map_err(|e| UrlError::Io(e))?
        .next()
        .ok_or(UrlError::InvalidUrl)
        .map(|a| format!("http://{}", a))
        .and_then(|s| {
            Url::parse(&s)
                .map_err(|e| UrlError::Parse(e))
        })
}

/// The error when loading the configuration.
#[derive(Debug)]
pub enum UrlError {
    /// The file could not be read.
    Io(std::io::Error),
    /// The file could not be parsed.
    InvalidUrl,
    Parse(url::ParseError),
}

impl Display for UrlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UrlError::Io(err) => write!(f, "{}", err),
            UrlError::InvalidUrl => write!(f, "{}", "Invalid URL"),
            UrlError::Parse(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for UrlError {}

/// A helper function to parse the TOML into a JSON.
pub fn convert(toml: Toml) -> Json {
    match toml {
        Toml::String(s) => Json::String(s),
        Toml::Integer(i) => Json::Number(i.into()),
        Toml::Float(f) => {
            let n = serde_json::Number::from_f64(f).expect("float infinite and nan not allowed");
            Json::Number(n)
        }
        Toml::Boolean(b) => Json::Bool(b),
        Toml::Array(arr) => Json::Array(arr.into_iter().map(convert).collect()),
        Toml::Table(table) => {
            Json::Object(table.into_iter().map(|(k, v)| (k, convert(v))).collect())
        }
        Toml::Datetime(dt) => Json::String(dt.to_string()),
    }
}
