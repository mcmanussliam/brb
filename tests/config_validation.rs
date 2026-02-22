use brb_cli::config::load_config_from_path;
use std::fs;
use tempfile::TempDir;

#[test]
fn validates_default_channel_references() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yml");
    fs::write(
        &config_path,
        r#"
version: 1
default_channels: [missing]
channels:
  desktop:
    type: desktop
"#,
    )
    .unwrap();

    let error = load_config_from_path(&config_path).unwrap_err().to_string();
    assert!(error.contains("default channel `missing`"));
}

#[test]
fn rejects_unknown_fields() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yml");
    fs::write(
        &config_path,
        r#"
version: 1
default_channels: [desktop]
channels:
  desktop:
    type: desktop
    unknown_field: true
"#,
    )
    .unwrap();

    let error = load_config_from_path(&config_path).unwrap_err().to_string();
    assert!(error.contains("unknown field"));
}

#[test]
fn rejects_unsupported_version() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yml");
    fs::write(
        &config_path,
        r#"
version: 2
default_channels: [desktop]
channels:
  desktop:
    type: desktop
"#,
    )
    .unwrap();

    let error = load_config_from_path(&config_path).unwrap_err().to_string();
    assert!(error.contains("unsupported version"));
}

#[test]
fn rejects_empty_default_channels() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yml");
    fs::write(
        &config_path,
        r#"
version: 1
default_channels: []
channels:
  desktop:
    type: desktop
"#,
    )
    .unwrap();

    let error = load_config_from_path(&config_path).unwrap_err().to_string();
    assert!(error.contains("default_channels must include at least one channel id"));
}
