<p align="center">
  <img src="assets/readme/demo.gif" alt="brb demo" />
</p>

# `brb`

Tired of babysitting commands for hours? `brb` is a wrapper for commands, it
waits for a given command to complete, then sends a completion event to your
configured channels.

Best used for long builds, test suites, data jobs, and deploy commands.

## Setup

Install from cargo:

```bash
cargo install brb-cli
```

Then setup like so:

```bash
# create global config
brb init

# check where config is stored
brb config path

# run something and notify default channels
brb cargo test

# override channels for one run
brb --channel phone pnpm test
```

## Usage

```plaintext
brb [--channel <channel-id> ...] <command> [args...]
brb init
brb channels list
brb channels validate
brb channels test <channel-id>
brb config
brb config path
brb --help
brb --version
```

If your wrapped command begins with flags, separate with `--`:

```bash
brb --channel desktop -- --version
```

## Cookbook

| Goal                      | Command                                               |
|---------------------------|-------------------------------------------------------|
| Run with defaults         | `brb cargo test`                                      |
| Use one specific channel  | `brb --channel desktop cargo test`                    |
| Use multiple channels     | `brb --channel mobile --channel ci-webhook pnpm test` |
| Validate config           | `brb channels validate`                               |
| Send test notification    | `brb channels test desktop`                           |
| Print config path         | `brb config path`                                     |

## Config

A fully commented example is included at `assets/examples/config.yml`.

### Channel Types

| Type       | Purpose                           | Required Fields | Optional Fields                         |
|------------|-----------------------------------|-----------------|-----------------------------------------|
| `desktop`  | Local desktop notification        | `type`          | none                                    |
| `webhook`  | HTTP JSON event delivery          | `type`, `url`   | `method` (default `POST`), `headers`    |
| `custom`   | Execute your own notifier process | `type`, `exec`  | `args`, `env`                           |

### Custom

`custom` channels are an executable that receive one JSON event on stdin.

`exec` supports:

- Relative path: resolved from the working directory where `brb` is launched.
- Absolute path: used directly.

The JSON the executable will receive is shaped like so:

```json
{
  "tool": "brb",
  "status": "success",
  "command": ["pnpm", "test"],
  "cwd": "/path/to/project",
  "started_at": "2026-02-22T12:00:00.000Z",
  "finished_at": "2026-02-22T12:00:03.250Z",
  "duration_ms": 3250,
  "exit_code": 0,
  "host": "my-machine"
}
```
