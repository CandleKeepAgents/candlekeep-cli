use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR: &str = ".candlekeep";
const CONFIG_FILE: &str = "config.toml";
const DEFAULT_API_URL: &str = "https://www.getcandlekeep.com";
const API_URL_ENV: &str = "CANDLEKEEP_API_URL";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub api: ApiConfig,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiConfig {
    pub url: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            url: DEFAULT_API_URL.to_string(),
        }
    }
}

/// Get the path to the config directory (~/.candlekeep)
pub fn config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(CONFIG_DIR))
}

/// Get the path to the config file (~/.candlekeep/config.toml)
pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join(CONFIG_FILE))
}

/// Load config from file, creating defaults if it doesn't exist
pub fn load_config() -> Result<Config> {
    let path = config_path()?;

    if !path.exists() {
        return Ok(Config::default());
    }

    let contents = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let config: Config =
        toml::from_str(&contents).with_context(|| "Failed to parse config file")?;

    Ok(config)
}

/// Save config to file, creating directory if needed
pub fn save_config(config: &Config) -> Result<()> {
    let dir = config_dir()?;
    let path = config_path()?;

    // Create directory if it doesn't exist
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create config directory: {}", dir.display()))?;
    }

    let contents = toml::to_string_pretty(config).context("Failed to serialize config")?;

    fs::write(&path, contents)
        .with_context(|| format!("Failed to write config file: {}", path.display()))?;

    Ok(())
}

/// Get the API key from config
pub fn get_api_key() -> Result<Option<String>> {
    let config = load_config()?;
    Ok(config.auth.api_key)
}

/// Get the API URL from environment variable or config
pub fn get_api_url() -> Result<String> {
    // Environment variable takes precedence
    if let Ok(url) = env::var(API_URL_ENV) {
        return Ok(url);
    }
    let config = load_config()?;
    Ok(config.api.url)
}

/// Save API key to config
pub fn save_api_key(api_key: &str) -> Result<()> {
    let mut config = load_config()?;
    config.auth.api_key = Some(api_key.to_string());
    save_config(&config)
}

/// Clear all credentials from config
pub fn clear_config() -> Result<()> {
    let mut config = load_config()?;
    config.auth.api_key = None;
    save_config(&config)
}

/// Check if user is authenticated
pub fn is_authenticated() -> bool {
    get_api_key().ok().flatten().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.auth.api_key.is_none());
        assert_eq!(config.api.url, DEFAULT_API_URL);
    }

    #[test]
    fn test_config_serialization() {
        let mut config = Config::default();
        config.auth.api_key = Some("ck_test123".to_string());

        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        assert_eq!(
            deserialized.auth.api_key,
            Some("ck_test123".to_string())
        );
    }
}
