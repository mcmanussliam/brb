use crate::config::{ChannelConfig, Config, CustomChannel, WebhookChannel};
use crate::event::CompletionEvent;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::process::{Command, Stdio};

/// Notification delivery status for a single channel.
#[derive(Debug, Clone)]
pub struct DeliveryResult {
    /// Channel id from config.
    pub channel_id: String,

    /// Whether delivery succeeded.
    pub success: bool,

    /// Optional failure reason.
    pub error: Option<String>,
}

/// Sends one event to all selected channel IDs.
pub fn notify_selected(
    config: &Config,
    selected_channel_ids: &[String],
    event: &CompletionEvent,
) -> Vec<DeliveryResult> {
    selected_channel_ids
        .iter()
        .map(|channel_id| {
            let Some(channel) = config.channels.get(channel_id) else {
                return DeliveryResult {
                    channel_id: channel_id.clone(),
                    success: false,
                    error: Some("channel not found in config".to_string()),
                };
            };

            match send_one(channel, event) {
                Ok(()) => DeliveryResult {
                    channel_id: channel_id.clone(),
                    success: true,
                    error: None,
                },
                Err(error) => DeliveryResult {
                    channel_id: channel_id.clone(),
                    success: false,
                    error: Some(redact_sensitive(&error)),
                },
            }
        })
        .collect()
}

fn send_one(channel: &ChannelConfig, event: &CompletionEvent) -> Result<(), String> {
    match channel {
        ChannelConfig::Desktop(_) => send_desktop(event),
        ChannelConfig::Webhook(webhook) => send_webhook(webhook, event),
        ChannelConfig::Custom(custom) => send_custom(custom, event),
    }
}

fn send_desktop(event: &CompletionEvent) -> Result<(), String> {
    let title = if event.exit_code == 0 {
        "brb: success".to_string()
    } else {
        format!("brb: failed (exit {})", event.exit_code)
    };

    let duration_s = event.duration_ms as f64 / 1000.0;
    let body = format!("{} ({:.2}s)", event.command.join(" "), duration_s);

    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "display notification \"{}\" with title \"{}\"",
            escape_applescript(&body),
            escape_applescript(&title)
        );

        let status = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .status()
            .map_err(|error| format!("failed to run osascript: {error}"))?;

        if status.success() {
            Ok(())
        } else {
            Err("desktop notifier command returned non-zero status".to_string())
        }
    }

    #[cfg(target_os = "linux")]
    {
        let status = Command::new("notify-send")
            .arg(title)
            .arg(body)
            .status()
            .map_err(|error| format!("failed to run notify-send: {error}"))?;

        if status.success() {
            Ok(())
        } else {
            Err("desktop notifier command returned non-zero status".to_string())
        }
    }

    #[cfg(target_os = "windows")]
    {
        let _ = title;
        let _ = body;
        Err("desktop channel is not implemented on Windows yet".to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = title;
        let _ = body;
        Err("desktop channel is not supported on this platform".to_string())
    }
}

fn send_webhook(webhook: &WebhookChannel, event: &CompletionEvent) -> Result<(), String> {
    let method = reqwest::Method::from_bytes(webhook.method.as_bytes())
        .map_err(|_| "invalid HTTP method in webhook config".to_string())?;
    let headers = build_headers(&webhook.headers)?;

    let client = reqwest::blocking::Client::new();
    let response = client
        .request(method, &webhook.url)
        .headers(headers)
        .json(event)
        .send()
        .map_err(|_| "webhook request failed".to_string())?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!(
            "webhook returned HTTP {}",
            response.status().as_u16()
        ))
    }
}

fn build_headers(
    raw_headers: &std::collections::BTreeMap<String, String>,
) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();

    for (key, value) in raw_headers {
        let name = HeaderName::try_from(key.as_str())
            .map_err(|_| format!("invalid webhook header name `{key}`"))?;
        let value = HeaderValue::try_from(value.as_str())
            .map_err(|_| format!("invalid value for webhook header `{key}`"))?;
        headers.insert(name, value);
    }

    Ok(headers)
}

fn send_custom(custom: &CustomChannel, event: &CompletionEvent) -> Result<(), String> {
    let mut command = Command::new(&custom.exec);
    command
        .args(&custom.args)
        .envs(&custom.env)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|_| format!("failed to start custom notifier `{}`", custom.exec))?;

    let payload =
        serde_json::to_vec(event).map_err(|_| "failed to encode event payload".to_string())?;
    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin
            .write_all(&payload)
            .map_err(|_| "failed writing event payload to custom notifier".to_string())?;
    }

    let output = child
        .wait_with_output()
        .map_err(|_| "failed waiting for custom notifier process".to_string())?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        Err("custom notifier exited with non-zero status".to_string())
    } else {
        Err(format!(
            "custom notifier failed: {}",
            truncate_for_error(&stderr, 200)
        ))
    }
}

fn truncate_for_error(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }

    let truncated = input.chars().take(max_chars).collect::<String>();
    format!("{truncated}...")
}

/// It sounds stupid but odds are people are accidientally going to send down
/// sensitive data, we can try our best from out side to prevent it from making it
/// to the channel.
fn redact_sensitive(input: &str) -> String {
    let mut output = input.to_string();

    let patterns = [
        (
            Regex::new(r"(?i)(authorization\s*[:=]\s*bearer\s+)[^\s]+")
                .expect("valid authorization redaction regex"),
            "$1[REDACTED]",
        ),
        (
            Regex::new(r"(?i)((?:token|secret|password|api[_-]?key)\s*[:=]\s*)[^\s,;]+")
                .expect("valid token redaction regex"),
            "$1[REDACTED]",
        ),
        (
            Regex::new(r"(?i)(https?://[^/\s:@]+:)[^@\s/]+@")
                .expect("valid url credentials redaction regex"),
            "$1[REDACTED]@",
        ),
        (
            Regex::new(r"(?i)([?&](?:token|key|secret|sig)=)[^&\s]+")
                .expect("valid query secret redaction regex"),
            "$1[REDACTED]",
        ),
    ];

    for (pattern, replacement) in patterns {
        output = pattern.replace_all(&output, replacement).to_string();
    }

    output
}

#[cfg(target_os = "macos")]
fn escape_applescript(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}
