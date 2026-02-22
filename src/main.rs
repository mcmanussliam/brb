use brb_cli::channels::{DeliveryResult, notify_selected};
use brb_cli::cli::{Action, ChannelsAction, ConfigAction, RunArgs, parse_args, usage};
use brb_cli::config::{ConfigError, InitStatus, config_file_path, init_config, load_config};
use brb_cli::event::CompletionEvent;
use brb_cli::runner::run_command;
use thiserror::Error;

#[derive(Debug, Error)]
enum AppError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Cli(#[from] brb_cli::cli::CliError),
    #[error(transparent)]
    Config(#[from] ConfigError),
}

fn main() {
    let code = match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("brb: {error}");
            1
        }
    };

    std::process::exit(code);
}

fn run() -> Result<i32, AppError> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let action = parse_args(args)?;

    match action {
        Action::Help => {
            println!("{}", usage());
            Ok(0)
        }
        Action::Version => {
            println!("brb {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
        Action::Init => handle_init(),
        Action::Channels(action) => handle_channels(action),
        Action::Config(action) => handle_config(action),
        Action::Run(args) => handle_run(args),
    }
}

fn handle_init() -> Result<i32, AppError> {
    match init_config()? {
        InitStatus::Created(path) => {
            println!("brb: created config at {}", path.display());
            Ok(0)
        }
        InitStatus::AlreadyExists(path) => {
            println!("brb: config already exists at {}", path.display());
            Ok(0)
        }
    }
}

fn handle_channels(action: ChannelsAction) -> Result<i32, AppError> {
    let loaded = load_config()?;

    match action {
        ChannelsAction::List => {
            println!("Config: {}", loaded.path.display());
            println!("Channels:");
            for (channel_id, channel) in &loaded.config.channels {
                let default_label = if loaded.config.default_channels.contains(channel_id) {
                    " (default)"
                } else {
                    ""
                };
                println!(
                    "- {} [{}]{}",
                    channel_id,
                    channel.type_name(),
                    default_label
                );
            }
            Ok(0)
        }
        ChannelsAction::Validate => {
            println!("brb: config is valid ({})", loaded.path.display());
            Ok(0)
        }
        ChannelsAction::Test { channel_id } => {
            if !loaded.config.channels.contains_key(&channel_id) {
                return Err(AppError::Message(format!(
                    "channel `{channel_id}` is not defined in config"
                )));
            }

            let event = CompletionEvent::test_event();
            let results =
                notify_selected(&loaded.config, std::slice::from_ref(&channel_id), &event);
            let result = &results[0];

            if result.success {
                println!("brb: test notification delivered on `{channel_id}`");
                Ok(0)
            } else {
                let reason = result
                    .error
                    .as_deref()
                    .unwrap_or("unknown notification error");
                eprintln!("brb: test notification failed on `{channel_id}`: {reason}");
                Ok(1)
            }
        }
    }
}

fn handle_config(action: ConfigAction) -> Result<i32, AppError> {
    match action {
        ConfigAction::Path => {
            let path = config_file_path()?;
            println!("{}", path.display());
            Ok(0)
        }
    }
}

fn handle_run(args: RunArgs) -> Result<i32, AppError> {
    let loaded = load_config()?;
    let selected_channels = resolve_channels(&loaded.config.default_channels, &args.channels)?;
    for channel_id in &selected_channels {
        if !loaded.config.channels.contains_key(channel_id) {
            return Err(AppError::Message(format!(
                "selected channel `{channel_id}` is not defined in config"
            )));
        }
    }

    let run = run_command(&args.command);
    if let Some(error) = &run.spawn_error {
        eprintln!("brb: {error}");
    }

    let event = CompletionEvent::from_run(&run);
    let results = notify_selected(&loaded.config, &selected_channels, &event);
    print_summary(run.exit_code, &results);

    Ok(run.exit_code)
}

fn resolve_channels(
    default_channels: &[String],
    explicit_channels: &[String],
) -> Result<Vec<String>, AppError> {
    let channels = if explicit_channels.is_empty() {
        default_channels.to_vec()
    } else {
        explicit_channels.to_vec()
    };

    if channels.is_empty() {
        return Err(AppError::Message(
            "no channels selected (set default_channels or pass --channel)".to_string(),
        ));
    }

    Ok(channels)
}

fn print_summary(exit_code: i32, results: &[DeliveryResult]) {
    let total = results.len();
    let sent = results.iter().filter(|result| result.success).count();
    let failed = results
        .iter()
        .filter(|result| !result.success)
        .map(|result| {
            let reason = result
                .error
                .as_deref()
                .unwrap_or("unknown notification error");
            format!("{} ({reason})", result.channel_id)
        })
        .collect::<Vec<_>>();

    let command_label = if exit_code == 0 {
        "command succeeded"
    } else {
        "command failed"
    };

    if failed.is_empty() {
        eprintln!("brb: {command_label} (exit {exit_code}); notifications sent {sent}/{total}");
    } else {
        eprintln!(
            "brb: {command_label} (exit {exit_code}); notifications sent {sent}/{total}; failed: {}",
            failed.join(", ")
        );
    }
}
