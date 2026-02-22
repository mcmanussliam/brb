use chrono::{DateTime, Utc};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Captured result from executing a wrapped command.
#[derive(Debug, Clone)]
pub struct RunResult {
    /// Command argv used for execution.
    pub command: Vec<String>,

    /// UTC timestamp for command start.
    pub started_at: DateTime<Utc>,

    /// UTC timestamp for command finish.
    pub finished_at: DateTime<Utc>,

    /// Total command execution duration.
    pub duration: Duration,

    /// Final process exit code (`127` when spawn fails).
    pub exit_code: i32,

    /// Spawn-time error message if the command failed to start.
    pub spawn_error: Option<String>,
}

/// Runs a command with inherited stdio and returns completion metadata.
pub fn run_command(command: &[String]) -> RunResult {
    let started_at = Utc::now();
    let started = Instant::now();

    if command.is_empty() {
        let finished_at = Utc::now();
        return RunResult {
            command: vec![],
            started_at,
            finished_at,
            duration: started.elapsed(),
            exit_code: 2,
            spawn_error: Some("no command provided".to_string()),
        };
    }

    let status = Command::new(&command[0])
        .args(&command[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(status) => {
            let finished_at = Utc::now();
            RunResult {
                command: command.to_vec(),
                started_at,
                finished_at,
                duration: started.elapsed(),
                exit_code: status.code().unwrap_or(1),
                spawn_error: None,
            }
        }
        Err(error) => {
            let finished_at = Utc::now();
            RunResult {
                command: command.to_vec(),
                started_at,
                finished_at,
                duration: started.elapsed(),
                exit_code: 127,
                spawn_error: Some(format!("failed to start `{}`: {error}", command[0])),
            }
        }
    }
}
