use crate::runner::RunResult;
use chrono::SecondsFormat;
use serde::Serialize;
use std::env;

/// Serialized payload sent to webhook/custom channels.
#[derive(Debug, Clone, Serialize)]
pub struct CompletionEvent {
    /// Constant tool identifier.
    pub tool: String,

    /// `success` when exit code is 0, otherwise `failure`.
    pub status: String,

    /// Command argv.
    pub command: Vec<String>,

    /// Working directory where `brb` was invoked.
    pub cwd: String,

    /// UTC start timestamp (RFC3339).
    pub started_at: String,

    /// UTC finish timestamp (RFC3339).
    pub finished_at: String,

    /// Total duration in milliseconds.
    pub duration_ms: u128,

    /// Wrapped command exit code.
    pub exit_code: i32,

    /// Hostname when available.
    pub host: String,
}

impl CompletionEvent {
    /// Builds a completion event from a finished wrapped command.
    pub fn from_run(run: &RunResult) -> Self {
        let cwd = env::current_dir()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| ".".to_string());

        let host = hostname::get()
            .ok()
            .map(|name| name.to_string_lossy().to_string())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "unknown-host".to_string());

        Self {
            tool: "brb".to_string(),
            status: if run.exit_code == 0 {
                "success".to_string()
            } else {
                "failure".to_string()
            },
            command: run.command.clone(),
            cwd,
            started_at: run.started_at.to_rfc3339_opts(SecondsFormat::Millis, true),
            finished_at: run.finished_at.to_rfc3339_opts(SecondsFormat::Millis, true),
            duration_ms: run.duration.as_millis(),
            exit_code: run.exit_code,
            host,
        }
    }

    /// Creates a synthetic event used by `brb channels test`.
    pub fn test_event() -> Self {
        let run = RunResult {
            command: vec![
                "brb".to_string(),
                "channels".to_string(),
                "test".to_string(),
            ],
            started_at: chrono::Utc::now(),
            finished_at: chrono::Utc::now(),
            duration: std::time::Duration::from_millis(1),
            exit_code: 0,
            spawn_error: None,
        };
        Self::from_run(&run)
    }
}
