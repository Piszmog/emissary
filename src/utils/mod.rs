use std::net::ToSocketAddrs;

use serde_json::Value as Json;
use toml::Value as Toml;
use url::Url;

pub fn to_url(addr: String, port: u16) -> Url {
    let forward_socket_addr = (addr, port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .expect("given forwarding address was not valid");

    let forward_url = format!("http://{}", forward_socket_addr);
    Url::parse(&forward_url).unwrap()
}

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
