use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Read, net::IpAddr};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub config: HashMap<String, Record>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub port: u16,
    pub multicast_groups: Vec<IpAddr>,
    pub destinations: Vec<String>,
}

impl Config {
    const FILENAME: &'static str = "config.toml";

    pub fn parse(mut filename: &str) -> Result<Config, String> {
        if filename.is_empty() {
            filename = Config::FILENAME;
        }

        let mut f = File::open(filename).map_err(|e| format!("error in opening file: {}", e))?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)
            .map_err(|e| format!("error in reading file contents: {}", e))?;

        let config: Config = toml::from_str(&contents)
            .map_err(|e| format!("error in parsing config: {}", e.message()))?;

        Ok(config)
    }
}
