# Config

`brb` uses a single global YAML config file. You can use this command to print
the exact path on your machine:

```bash
brb config

# also for your convenience that means you can easily:
nvim "$(brb config)"
nano "$(brb config)"
code "$(brb config)"
```

Run this to validate config loading and parsing:

```bash
brb channels validate
```

## File Shape

```yml
version: 1
default_channels:
  - desktop
channels:
  desktop:
    type: desktop
```

## Top-Level Fields

| Field | Type | Required | Notes |
|---|---|---|---|
| `version` | integer | yes | Currently 1. |
| `default_channels` | list of string | yes | Must include at least one existing channel ID. |
| `channels` | map | yes | Map of channel ID to channel config; must not be empty. |

## Channel Types

| Type | Purpose | Required Fields | Optional Fields |
|---|---|---|---|
| `desktop` | Local desktop notification | `type` | none |
| `webhook` | HTTP JSON event delivery | `type`, `url` | `method` (default `POST`), `headers` |
| `custom` | Execute your own notifier process | `type`, `exec` | `args`, `env` |

Unknown fields are rejected.

## Environment Interpolation

Any channel string value can include `${env:VAR_NAME}`.

Example:

```yml
url: https://example.com/hook?token=${env:BRB_TOKEN}
```

Rules:

- Missing environment variables cause config load failure.
- Invalid expressions (like `${env:}`) cause config load failure.
- Interpolation applies to webhook fields (`url`, `method`, headers) and custom fields (`exec`, `args`, `env` values).

## Webhook Behavior

For `type: webhook`:

- `method` defaults to `POST` if omitted.
- `headers` are optional.
- `brb` sends the completion event as JSON body.
- Non-2xx responses are treated as delivery failures.

## Custom Behavior

For `type: custom`:

- `exec` can be an executable name, relative path, or absolute path.
- `args` are passed as command-line arguments.
- `env` adds/overrides environment variables for the child process.
- `brb` writes exactly one JSON completion event to the notifier process stdin.
- Child stdout is discarded; stderr is captured for error reporting.

Example shell notifier: `assets/examples/scripts/write-to-logs.sh`

## Full Example

```yml
version: 1

# channel ids used when you run `brb <command>` without `--channel`.
# every ID here must exist under `channels`.
default_channels:
  - desktop

# all channels keyed by your own channel id
channels:
  # minimal local notification channel.
  # works without any extra fields.
  desktop:
    type: desktop

  # generic webhook example
  ci-webhook:
    type: webhook # channel type: desktop | webhook | custom

    # target URL for webhook delivery.
    # `${env:VAR}` reads from environment variables at runtime.
    url: ${env:BRB_WEBHOOK_URL}

    method: POST # optional HTTP method (defaults to POST if omitted).

    # optional request headers.
    headers:
      Authorization: Bearer ${env:BRB_WEBHOOK_TOKEN}
      Content-Type: application/json

  # custom executable channel example.
  write:
    type: custom

    # executable to be run on notify
    # relative path `./scripts/notifier.sh` relative to where `brb` is run
    # absolute path: used directly
    exec: ./abs/path/to/examples/write-to-logs.sh

    args: [] # optional cli args passed to `exec`.

    # optional environment overrides for the child process.
    env:
      BRB_CUSTOM_LOG_FILE: /tmp/brb-custom-channel.log
```

For the generated starter config, see `assets/default-config.yml`.
