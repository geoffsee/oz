use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api_url: String,
    pub api_key: Option<String>,
}

impl Config {
    pub fn path() -> Result<PathBuf> {
        let dir = dirs::config_dir().context("config dir")?.join("ozzy");
        Ok(dir.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(&path)?;
        toml::from_str(&text).context("parse config")
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn clear() -> Result<()> {
        let path = Self::path()?;
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: std::env::var("OZZY_API_URL").unwrap_or_else(|_| "http://localhost:8787".into()),
            api_key: std::env::var("OZZY_API_KEY").ok(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_default() {
        std::env::set_var("OZZY_API_URL", "http://test-url");
        std::env::set_var("OZZY_API_KEY", "test-key");
        let default_config = Config::default();
        assert_eq!(default_config.api_url, "http://test-url");
        assert_eq!(default_config.api_key, Some("test-key".to_string()));

        std::env::remove_var("OZZY_API_URL");
        std::env::remove_var("OZZY_API_KEY");
        let default_config2 = Config::default();
        assert_eq!(default_config2.api_url, "http://localhost:8787");
        assert_eq!(default_config2.api_key, None);
    }

    #[test]
    fn test_config_save_load_clear() {
        let temp = tempdir().unwrap();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", temp.path());

        let path = Config::path().unwrap();
        assert!(path.to_string_lossy().contains("ozzy"));

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.api_url, "http://localhost:8787");

        let my_config = Config {
            api_url: "http://my-api-url".to_string(),
            api_key: Some("my-api-key".to_string()),
        };
        my_config.save().unwrap();

        let loaded2 = Config::load().unwrap();
        assert_eq!(loaded2.api_url, "http://my-api-url");
        assert_eq!(loaded2.api_key, Some("my-api-key".to_string()));

        Config::clear().unwrap();
        assert!(!path.exists());

        if let Some(home) = old_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
    }
}
