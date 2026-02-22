use directories::BaseDirs;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// The top-level YAML configuration structure.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Config schema version.
    pub version: u32,

    /// Channel IDs used when `--channel` is omitted.
    #[serde(default)]
    pub default_channels: Vec<String>,

    /// Channel definitions keyed by channel ID.
    pub channels: BTreeMap<String, ChannelConfig>,
}

/// A single channel definition.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ChannelConfig {
    /// Local desktop notification channel.
    Desktop(DesktopChannel),

    /// Generic webhook channel.
    Webhook(WebhookChannel),

    /// External command-based custom channel.
    Custom(CustomChannel),
}

/// Configuration for `type: desktop`.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DesktopChannel {}

/// Configuration for `type: webhook`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WebhookChannel {
    /// Target URL for webhook delivery.
    pub url: String,

    /// HTTP method (defaults to POST).
    #[serde(default = "default_http_method")]
    pub method: String,

    /// Optional HTTP headers.
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
}

/// Configuration for `type: custom`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomChannel {
    /// Executable name or path.
    pub exec: String,

    /// Optional command-line arguments.
    #[serde(default)]
    pub args: Vec<String>,

    /// Optional environment variable overrides.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

/// Fully-loaded config plus where it came from.
#[derive(Debug, Clone)]
pub struct LoadedConfig {
    /// Absolute file path used for loading.
    pub path: PathBuf,

    /// Parsed and validated config.
    pub config: Config,
}

/// Result of running `brb init`.
#[derive(Debug, Clone)]
pub enum InitStatus {
    /// Config file was created.
    Created(PathBuf),

    /// Config file already existed.
    AlreadyExists(PathBuf),
}

/// Config loading/validation failures.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("unable to determine user config directory")]
    NoConfigDirectory,
    #[error("config file not found: {0}")]
    NotFound(String),
    #[error("failed to read config file: {0}")]
    ReadFailed(#[from] std::io::Error),
    #[error("invalid YAML config: {0}")]
    ParseFailed(#[from] serde_yaml::Error),
    #[error("missing environment variable for interpolation: {0}")]
    MissingEnvironmentVariable(String),
    #[error("invalid environment interpolation expression in config value: {0}")]
    InvalidInterpolation(String),
    #[error("invalid config: {0}")]
    InvalidConfig(String),
}

impl ChannelConfig {
    /// Returns stable type label for display output.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Desktop(_) => "desktop",
            Self::Webhook(_) => "webhook",
            Self::Custom(_) => "custom",
        }
    }
}

/// Loads config from the global `config.yml` and validates it.
pub fn load_config() -> Result<LoadedConfig, ConfigError> {
    let path = config_file_path()?;
    if !path.exists() {
        return Err(ConfigError::NotFound(path.display().to_string()));
    }

    let raw = fs::read_to_string(&path)?;
    let mut config: Config = serde_yaml::from_str(&raw)?;
    interpolate_env_values(&mut config)?;
    validate_config(&config)?;

    Ok(LoadedConfig { path, config })
}

/// Creates a default global config file if it does not already exist.
pub fn init_config() -> Result<InitStatus, ConfigError> {
    let path = config_file_path()?;
    if path.exists() {
        return Ok(InitStatus::AlreadyExists(path));
    }

    let parent = path.parent().ok_or_else(|| {
        ConfigError::InvalidConfig("calculated config path has no parent directory".to_string())
    })?;

    fs::create_dir_all(parent)?;
    fs::write(&path, default_config_yaml())?;

    Ok(InitStatus::Created(path))
}

/// Returns the absolute path to the global config file.
pub fn config_file_path() -> Result<PathBuf, ConfigError> {
    let base_dirs = BaseDirs::new().ok_or(ConfigError::NoConfigDirectory)?;
    Ok(base_dirs.config_dir().join("brb").join("config.yml"))
}

/// Validates static schema and cross-field constraints.
pub fn validate_config(config: &Config) -> Result<(), ConfigError> {
    if config.version != 1 {
        return Err(ConfigError::InvalidConfig(format!(
            "unsupported version {}; expected 1",
            config.version
        )));
    }

    if config.channels.is_empty() {
        return Err(ConfigError::InvalidConfig(
            "at least one channel must be configured".to_string(),
        ));
    }

    if config.default_channels.is_empty() {
        return Err(ConfigError::InvalidConfig(
            "default_channels must include at least one channel id".to_string(),
        ));
    }

    for channel_id in &config.default_channels {
        if !config.channels.contains_key(channel_id) {
            return Err(ConfigError::InvalidConfig(format!(
                "default channel `{channel_id}` is not defined in channels"
            )));
        }
    }

    Ok(())
}

fn default_http_method() -> String {
    "POST".to_string()
}

fn interpolate_env_values(config: &mut Config) -> Result<(), ConfigError> {
    for channel in config.channels.values_mut() {
        match channel {
            ChannelConfig::Desktop(_) => {}
            ChannelConfig::Webhook(webhook) => {
                webhook.url = interpolate_env(&webhook.url)?;
                webhook.method = interpolate_env(&webhook.method)?;
                for value in webhook.headers.values_mut() {
                    *value = interpolate_env(value)?;
                }
            }
            ChannelConfig::Custom(custom) => {
                custom.exec = interpolate_env(&custom.exec)?;
                for arg in &mut custom.args {
                    *arg = interpolate_env(arg)?;
                }
                for value in custom.env.values_mut() {
                    *value = interpolate_env(value)?;
                }
            }
        }
    }

    Ok(())
}

fn interpolate_env(value: &str) -> Result<String, ConfigError> {
    let mut output = String::new();
    let mut rest = value;

    loop {
        let Some(start) = rest.find("${env:") else {
            output.push_str(rest);
            break;
        };

        output.push_str(&rest[..start]);
        let placeholder = &rest[start + 6..];
        let Some(end) = placeholder.find('}') else {
            return Err(ConfigError::InvalidInterpolation(value.to_string()));
        };

        let env_name = &placeholder[..end];
        if env_name.is_empty() {
            return Err(ConfigError::InvalidInterpolation(value.to_string()));
        }

        let env_value = std::env::var(env_name)
            .map_err(|_| ConfigError::MissingEnvironmentVariable(env_name.to_string()))?;
        output.push_str(&env_value);
        rest = &placeholder[end + 1..];
    }

    Ok(output)
}

fn default_config_yaml() -> &'static str {
    include_str!("../assets/default-config.yml")
}

/// Loads and validates config from an explicit file path.
///
/// This helper is used by integration tests.
pub fn load_config_from_path(path: &Path) -> Result<Config, ConfigError> {
    let raw = fs::read_to_string(path)?;
    let mut config: Config = serde_yaml::from_str(&raw)?;
    interpolate_env_values(&mut config)?;
    validate_config(&config)?;
    Ok(config)
}
