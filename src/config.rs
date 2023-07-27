use log::debug;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs::File, io::Read};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub mdns: Vec<MdnsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdnsConfig {
    pub destinations: Vec<String>,
    pub sources: Vec<String>,
    pub filters: HashSet<String>,
}

impl Config {
    const FILENAME: &'static str = "config.toml";

    pub fn parse(mut filename: &str) -> Result<Config, String> {
        if filename.is_empty() {
            debug!(
                "filename is empty. using default name: {}",
                Config::FILENAME
            );
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
