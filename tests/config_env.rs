use brb_cli::config::{ChannelConfig, load_config_from_path};
use std::fs;
use tempfile::TempDir;

#[test]
fn interpolates_environment_values() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yml");
    fs::write(
        &config_path,
        r#"
version: 1
default_channels: [ci-webhook]
channels:
  ci-webhook:
    type: webhook
    url: https://example.com/hook?token=${env:PATH}
"#,
    )
    .unwrap();

    let config = load_config_from_path(&config_path).unwrap();
    let channel = config.channels.get("ci-webhook").unwrap();
    let ChannelConfig::Webhook(webhook) = channel else {
        panic!("expected webhook channel");
    };
    assert!(webhook.url.starts_with("https://example.com/hook?token="));
    assert!(!webhook.url.contains("${env:PATH}"));
}

#[test]
fn missing_environment_variable_is_error() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yml");
    fs::write(
        &config_path,
        r#"
version: 1
default_channels: [ci-webhook]
channels:
  ci-webhook:
    type: webhook
    url: https://example.com/hook?token=${env:BRB_TEST_MISSING_ENV_12345}
"#,
    )
    .unwrap();

    let error = load_config_from_path(&config_path).unwrap_err().to_string();
    assert!(error.contains("missing environment variable"));
}

#[test]
fn invalid_interpolation_expression_is_error() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yml");
    fs::write(
        &config_path,
        r#"
version: 1
default_channels: [ci-webhook]
channels:
  ci-webhook:
    type: webhook
    url: https://example.com/hook?token=${env:}
"#,
    )
    .unwrap();

    let error = load_config_from_path(&config_path).unwrap_err().to_string();
    assert!(error.contains("invalid environment interpolation expression"));
}
