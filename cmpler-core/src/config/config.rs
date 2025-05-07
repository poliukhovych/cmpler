use inkwell::OptimizationLevel;
use serde::{Deserialize, Serialize};
use std::{env, fs};
use std::path::{Path, PathBuf};
use crate::config::ConfigError;

/// Main settings, can be overridden in config file `cmpler.toml`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// LLVM optimization level
    #[serde(default = "default_opt_level", skip_serializing, skip_deserializing)] // TODO: find alternative ways
    pub opt_level: OptimizationLevel,
    /// Output of IR
    #[serde(default)]
    pub emit_ir: bool,
    /// Output of obj code
    #[serde(default)]
    pub emit_obj: bool,
    /// Target
    #[serde(default)]
    pub target: Option<String>,
    /// Path to source
    #[serde(default)]
    pub output: Option<PathBuf>,
    /// Detailed log
    #[serde(default)]
    pub verbose: bool,
}

fn default_opt_level() -> OptimizationLevel {
    OptimizationLevel::Default
}

impl Default for Config {
    fn default() -> Self {
        Config {
            opt_level: default_opt_level(),
            emit_ir: false,
            emit_obj: false,
            target: None,
            output: None,
            verbose: false,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        if let Some(path) = find_config_file("cmpler.toml") {
            let text = fs::read_to_string(&path)?;
            let cfg: Config = toml::from_str(&text)?;
            Ok(cfg)
        } else {
            Ok(Config::default())
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let text = fs::read_to_string(path)?;
        let cfg: Config = toml::from_str(&text)?;
        Ok(cfg)
    }
}

fn find_config_file(name: &str) -> Option<PathBuf> {
    let mut dir = env::current_dir().ok()?;
    loop {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}
