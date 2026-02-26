# Getting Started

## Install

```bash
cargo install brb-cli
```

Check the install:

```bash
brb --version
```

## Initialise Config

Create a default config file:

```bash
brb init
```

Print the active config path:

```bash
brb config path
```

## Run Your First Command

```bash
brb cargo test
```

This uses channels listed under `default_channels` in your config, by default this
is just `desktop`.

## Channels

### Override Channels Per Run

Use repeated `--channel` flags to override defaults for a single invocation:

```bash
brb --channel desktop --channel ci-webhook pnpm test
```

### Channel Rules

- If at least one `--channel` is provided, `brb` uses only those channel IDs.
- If no `--channel` is provided, `brb` uses `default_channels`.
- If any selected channel ID does not exist, `brb` exits with an error.

## Cookbook

| Goal | Command |
|---|---|
| Run with defaults | `brb cargo test` |
| Use one specific channel | `brb --channel desktop cargo test` |
| Use multiple channels | `brb --channel desktop --channel ci-webhook pnpm test` |
| Validate config | `brb channels validate` |
| Send test notification | `brb channels test desktop` |
| Print config path | `brb config path` |
