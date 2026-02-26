# Command Reference

## Summary

```text
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

## Commands

### `brb <command> [args...]`

Runs the command and sends completion notifications.

Behavior:

- Loads config from the global config path.
- Resolves selected channels (`--channel` overrides defaults).
- Runs wrapped command with inherited stdio.
- Attempts delivery to all selected channels.
- Exits with wrapped command exit code.

### `brb init`

Creates a default config file when one does not already exist.

### `brb channels list`

Prints configured channel IDs, types, and default marker.

### `brb channels validate`

Loads and validates config, then exits.

### `brb channels test <channel-id>`

Sends a synthetic success event to a single configured channel.

### `brb config`

Alias of `brb config path`.

### `brb config path`

Prints the config file path.

## Flags

### `--channel <channel-id>`

Repeatable. Selects channels for this run only.

```bash
brb --channel desktop --channel ci-webhook cargo test
```

If used, defaults are ignored.

### `--`

Separates `brb` flags from wrapped command flags.

```bash
brb --channel desktop -- --version
```
