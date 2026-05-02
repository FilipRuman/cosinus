use anyhow::{Context, Result};

use crate::linker::LinkerSettings;
use std::{fs, path::PathBuf};

impl LinkerSettings {
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path).context("failed to read file")?;
        toml::from_str(&content).context("failed to parse TOML")
    }
}
