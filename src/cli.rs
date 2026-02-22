use clap::{ArgAction, Command, CommandFactory, FromArgMatches, Parser, Subcommand};
use thiserror::Error;

/// High-level action parsed from CLI arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Initialise global config.
    Init,

    /// Run a channels management subcommand.
    Channels(ChannelsAction),

    /// Run a config management subcommand.
    Config(ConfigAction),

    /// Run a wrapped command.
    Run(RunArgs),

    /// Print help text.
    Help,

    /// Print version text.
    Version,
}

/// Command execution arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunArgs {
    /// Explicit channel IDs requested by repeated `--channel` flags.
    pub channels: Vec<String>,

    /// Command and arguments to execute.
    pub command: Vec<String>,
}

/// `brb channels` subcommands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelsAction {
    /// List configured channels.
    List,

    /// Validate config.
    Validate,

    /// Send a test notification to one channel.
    Test { channel_id: String },
}

/// `brb config` subcommands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigAction {
    /// Print config file path.
    Path,
}

/// CLI parsing errors for invalid user input.
#[derive(Debug, Error)]
pub enum CliError {
    #[error("`brb channels test` requires a <channel-id>")]
    MissingChannelId,
    #[error("`--channel` requires a value")]
    MissingChannelFlagValue,
    #[error("no command provided")]
    MissingCommand,
    #[error("{0}")]
    Clap(String),
}

#[derive(Debug, Parser)]
#[command(
    name = "brb",
    about = "run a command and notify when it completes",
    long_about = None,
    disable_help_subcommand = true
)]
struct CliArgs {
    /// Repeated channel override for wrapped command execution.
    #[arg(long = "channel", value_name = "channel-id", action = ArgAction::Append)]
    channels: Vec<String>,

    /// Built-in management subcommands.
    #[command(subcommand)]
    subcommand: Option<CliCommand>,

    /// Wrapped command and args.
    #[arg(
        value_name = "command",
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    command: Vec<String>,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    /// Initialise global config.
    Init,

    /// Run channels management commands.
    Channels {
        #[command(subcommand)]
        action: Option<CliChannelsAction>,
    },

    /// Run config management commands.
    Config {
        #[command(subcommand)]
        action: Option<CliConfigAction>,
    },
}

#[derive(Debug, Subcommand)]
enum CliChannelsAction {
    /// List configured channels.
    List,

    /// Validate config.
    Validate,

    /// Send a test notification to one channel.
    Test {
        /// Channel identifier.
        #[arg(value_name = "channel-id")]
        channel_id: String,
    },
}

#[derive(Debug, Subcommand)]
enum CliConfigAction {
    /// Print config file path.
    Path,
}

/// Returns clap-generated help text.
pub fn usage() -> String {
    cli_command().render_long_help().to_string()
}

/// Parses CLI args into a structured action.
pub fn parse_args(args: Vec<String>) -> Result<Action, CliError> {
    if args.is_empty() {
        return Ok(Action::Help);
    }

    let first = args[0].as_str();
    if matches!(first, "-h" | "--help") {
        return Ok(Action::Help);
    }

    if matches!(first, "-V" | "--version") {
        return Ok(Action::Version);
    }

    if first == "channels"
        && args.get(1).map(String::as_str) == Some("test")
        && args.get(2).is_none()
    {
        return Err(CliError::MissingChannelId);
    }

    if args.last().map(String::as_str) == Some("--channel") {
        return Err(CliError::MissingChannelFlagValue);
    }

    let mut argv = Vec::with_capacity(args.len() + 1);
    argv.push("brb".to_string());
    argv.extend(args);

    let matches = cli_command()
        .try_get_matches_from(argv)
        .map_err(|error| CliError::Clap(error.to_string()))?;
    let parsed =
        CliArgs::from_arg_matches(&matches).map_err(|error| CliError::Clap(error.to_string()))?;

    if let Some(subcommand) = parsed.subcommand {
        return match subcommand {
            CliCommand::Init => Ok(Action::Init),
            CliCommand::Channels { action } => {
                let action = match action {
                    Some(CliChannelsAction::List) | None => ChannelsAction::List,
                    Some(CliChannelsAction::Validate) => ChannelsAction::Validate,
                    Some(CliChannelsAction::Test { channel_id }) => {
                        ChannelsAction::Test { channel_id }
                    }
                };
                Ok(Action::Channels(action))
            }
            CliCommand::Config { action } => {
                let action = match action {
                    Some(CliConfigAction::Path) | None => ConfigAction::Path,
                };
                Ok(Action::Config(action))
            }
        };
    }

    if parsed.command.is_empty() {
        return Err(CliError::MissingCommand);
    }

    Ok(Action::Run(RunArgs {
        channels: parsed.channels,
        command: parsed.command,
    }))
}

fn cli_command() -> Command {
    CliArgs::command().override_usage(include_str!("../assets/usage.txt").trim_end())
}
