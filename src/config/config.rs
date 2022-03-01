use std::fs;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub http: Http,
    pub proxy: Http,
}

#[derive(Debug, Deserialize)]
pub struct Http {
    pub address: String,
    pub port: u16,
}

// TODO read json and yaml config files

pub fn read_toml_file(file_path: String) -> Config {
    let content = fs::read_to_string(file_path).expect("Unable to read file");
    toml::from_str(&content).expect("Unable to parse file")
}