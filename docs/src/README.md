# Be Right Back

`brb` wraps a command, waits for it to finish, then sends a completion event to
one or more notification channels.

It is useful for long-running builds, tests, deploys, migrations, and data jobs
where you do not want to watch a terminal the whole time.

## How It Works

1. `brb` loads your global config.
2. It resolves channels (from `default_channels` or repeated `--channel` flags).
3. It runs your command with inherited stdin/stdout/stderr.
4. It builds a completion event (status, timing, exit code, cwd, host, command).
5. It delivers that event to each selected channel.
6. It exits with the same code as the wrapped command.

## Platform Support

| Channel   | Linux           | MacOS      | Windows       |
|-----------|-----------------|------------|---------------|
| `desktop` | Partial Support | Supported  | Not Supported |
| `webhook` | Supported       | Supported  | Supported     |
| `custom`  | Supported       | Supported  | Supported     |

If there are any developers who specialise in any specific operating systems and want to contribute, be my guest.
