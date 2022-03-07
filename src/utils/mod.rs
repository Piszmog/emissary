use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::net::ToSocketAddrs;

use serde_json::Value as Json;
use toml::Value as Toml;
use url::Url;

/// Converts the given string to a URL.
pub fn to_url(addr: String, port: u16) -> Result<Url, UrlError> {
    (addr, port)
        .to_socket_addrs()
        .map_err(UrlError::Io)?
        .next()
        .ok_or(UrlError::InvalidUrl)
        .map(|a| format!("http://{}", a))
        .and_then(|s| {
            Url::parse(&s)
                .map_err(UrlError::Parse)
        })
}

/// The error when building a URL.
#[derive(Debug)]
pub enum UrlError {
    /// The url could not be read.
    Io(std::io::Error),
    /// The url is invalid.
    InvalidUrl,
    /// The url could not be parsed.
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
pub fn to_json(toml: Toml) -> Json {
    match toml {
        Toml::String(s) => Json::String(s),
        Toml::Integer(i) => Json::Number(i.into()),
        Toml::Float(f) => {
            let n = serde_json::Number::from_f64(f).expect("float infinite and nan not allowed");
            Json::Number(n)
        }
        Toml::Boolean(b) => Json::Bool(b),
        Toml::Array(arr) => Json::Array(arr.into_iter().map(to_json).collect()),
        Toml::Table(table) => {
            Json::Object(table.into_iter().map(|(k, v)| (k, to_json(v))).collect())
        }
        Toml::Datetime(dt) => Json::String(dt.to_string()),
    }
}

/// The structure containing the plain logging configuration.
#[derive(Debug, Clone)]
pub struct Plain {
    pub data: BTreeMap<String, String>,
}

// convert toml to Plain
impl From<Toml> for Plain {
    fn from(toml: Toml) -> Self {
        let mut data: BTreeMap<String, String> = BTreeMap::new();
        if let Toml::Table(table) = toml {
            for (k, value) in table {
                match value {
                    Toml::String(s) => data.insert(k, s),
                    Toml::Integer(i) => data.insert(k, i.to_string()),
                    Toml::Float(f) => data.insert(k, f.to_string()),
                    Toml::Boolean(b) => data.insert(k, b.to_string()),
                    Toml::Datetime(dt) => data.insert(k, dt.to_string()),
                    _ => None,
                };
            }
        };
        Plain { data }
    }
}

impl Display for Plain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.data.iter().fold(Ok(()), |acc, (k, v)| {
            write!(f, "{}=\"{}\" ", k, v)?;
            acc
        })
    }
}
