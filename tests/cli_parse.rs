use brb_cli::cli::{Action, ChannelsAction, ConfigAction, RunArgs, parse_args};

#[test]
fn parse_default_run_command() {
    let parsed = parse_args(vec!["pnpm".into(), "test".into()]).unwrap();
    assert_eq!(
        parsed,
        Action::Run(RunArgs {
            channels: vec![],
            command: vec!["pnpm".into(), "test".into()]
        })
    );
}

#[test]
fn parse_channel_flags() {
    let parsed = parse_args(vec![
        "--channel".into(),
        "desktop".into(),
        "--channel=ci-webhook".into(),
        "cargo".into(),
        "test".into(),
    ])
    .unwrap();
    assert_eq!(
        parsed,
        Action::Run(RunArgs {
            channels: vec!["desktop".into(), "ci-webhook".into()],
            command: vec!["cargo".into(), "test".into()]
        })
    );
}

#[test]
fn parse_channels_subcommand() {
    let parsed = parse_args(vec!["channels".into(), "validate".into()]).unwrap();
    assert_eq!(parsed, Action::Channels(ChannelsAction::Validate));
}

#[test]
fn parse_double_dash_separator() {
    let parsed = parse_args(vec![
        "--channel".into(),
        "desktop".into(),
        "--".into(),
        "echo".into(),
        "hello".into(),
    ])
    .unwrap();
    assert_eq!(
        parsed,
        Action::Run(RunArgs {
            channels: vec!["desktop".into()],
            command: vec!["echo".into(), "hello".into()]
        })
    );
}

#[test]
fn parse_missing_channel_value_is_error() {
    let error = parse_args(vec!["--channel".into()])
        .unwrap_err()
        .to_string();
    assert!(error.contains("requires a value"));
}

#[test]
fn parse_missing_command_is_error() {
    let error = parse_args(vec!["--channel".into(), "desktop".into()])
        .unwrap_err()
        .to_string();
    assert!(error.contains("no command provided"));
}

#[test]
fn parse_channels_test_requires_channel_id() {
    let error = parse_args(vec!["channels".into(), "test".into()])
        .unwrap_err()
        .to_string();
    assert!(error.contains("requires a <channel-id>"));
}

#[test]
fn parse_config_defaults_to_path() {
    let parsed = parse_args(vec!["config".into()]).unwrap();
    assert_eq!(parsed, Action::Config(ConfigAction::Path));
}

#[test]
fn parse_config_path_subcommand() {
    let parsed = parse_args(vec!["config".into(), "path".into()]).unwrap();
    assert_eq!(parsed, Action::Config(ConfigAction::Path));
}
