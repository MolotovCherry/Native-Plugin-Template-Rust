use std::fs;
use std::path::Path;

use eyre::Result;
use serde::{Deserialize, Serialize};

/// Need to figure out how to make a proper config?
/// quicktype.io can help you by converting json to Rust
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub opt1: bool,
    pub opt2: String,
    // etc
}

impl Config {
    /// Load a config file
    /// If path doesn't exist, creates and saves default config
    /// otherwise loads what's already there
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        // if path doesn't exist, create default config,
        // save it, and return it
        if !path.exists() {
            let config = Self::default();

            let serialized = toml::to_string_pretty(&config)?;
            fs::write(path, serialized)?;
            return Ok(config);
        }

        let data = fs::read_to_string(path)?;
        let config = toml::from_str::<Self>(&data)?;

        Ok(config)
    }
}
