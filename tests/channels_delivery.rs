use brb_cli::channels::notify_selected;
use brb_cli::config::{ChannelConfig, Config, CustomChannel, DesktopChannel, WebhookChannel};
use brb_cli::event::CompletionEvent;
use std::collections::BTreeMap;

fn config_with_channel(channel_id: &str, channel: ChannelConfig) -> Config {
    let mut channels = BTreeMap::new();
    channels.insert(channel_id.to_string(), channel);
    Config {
        version: 1,
        default_channels: vec![channel_id.to_string()],
        channels,
    }
}

#[test]
fn missing_selected_channel_reports_failure() {
    let config = config_with_channel("desktop", ChannelConfig::Desktop(DesktopChannel {}));
    let event = CompletionEvent::test_event();
    let selected = vec!["missing".to_string()];

    let results = notify_selected(&config, &selected, &event);
    assert_eq!(results.len(), 1);
    assert!(!results[0].success);
    assert_eq!(results[0].channel_id, "missing");
    assert!(
        results[0]
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("channel not found")
    );
}

#[test]
fn invalid_webhook_method_fails_fast() {
    let config = config_with_channel(
        "bad-webhook",
        ChannelConfig::Webhook(WebhookChannel {
            url: "https://example.com/hook".to_string(),
            method: "NOT A METHOD".to_string(),
            headers: BTreeMap::new(),
        }),
    );
    let event = CompletionEvent::test_event();
    let selected = vec!["bad-webhook".to_string()];

    let results = notify_selected(&config, &selected, &event);
    assert_eq!(results.len(), 1);
    assert!(!results[0].success);
    assert!(
        results[0]
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("invalid HTTP method")
    );
}

#[cfg(unix)]
#[test]
fn custom_channel_success_path() {
    let config = config_with_channel(
        "custom-ok",
        ChannelConfig::Custom(CustomChannel {
            exec: "sh".to_string(),
            args: vec!["-c".to_string(), "cat >/dev/null; exit 0".to_string()],
            env: BTreeMap::new(),
        }),
    );
    let event = CompletionEvent::test_event();
    let selected = vec!["custom-ok".to_string()];

    let results = notify_selected(&config, &selected, &event);
    assert_eq!(results.len(), 1);
    assert!(results[0].success);
}

#[cfg(unix)]
#[test]
fn custom_channel_failure_redacts_token_like_values() {
    let config = config_with_channel(
        "custom-fail",
        ChannelConfig::Custom(CustomChannel {
            exec: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "cat >/dev/null; echo 'token=abc123' 1>&2; exit 1".to_string(),
            ],
            env: BTreeMap::new(),
        }),
    );
    let event = CompletionEvent::test_event();
    let selected = vec!["custom-fail".to_string()];

    let results = notify_selected(&config, &selected, &event);
    assert_eq!(results.len(), 1);
    assert!(!results[0].success);
    let message = results[0].error.as_deref().unwrap_or_default();
    assert!(message.contains("[REDACTED]"));
    assert!(!message.contains("abc123"));
}
