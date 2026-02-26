# Event Payload

`brb` sends a completion event to `webhook` and `custom` channels.

## JSON Shape

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

## Field Semantics

| Field | Type | Description |
|---|---|---|
| `tool` | string | Constant value: `brb`. |
| `status` | string | `success` when exit code is `0`, otherwise `failure`. |
| `command` | string array | Command argv that `brb` executed. |
| `cwd` | string | Working directory where `brb` was invoked. |
| `started_at` | string | UTC RFC3339 timestamp with milliseconds. |
| `finished_at` | string | UTC RFC3339 timestamp with milliseconds. |
| `duration_ms` | integer | Runtime duration in milliseconds. |
| `exit_code` | integer | Wrapped command exit code (`127` if spawn failed). |
| `host` | string | Hostname, or `unknown-host` if unavailable. |

## Delivery Semantics

- `brb` attempts delivery independently for each selected channel.
- A failure on one channel does not stop attempts on others.
- The final process exit code still matches the wrapped command.

## Redaction

For delivery error messages, `brb` attempts to redact token-like values (for
example bearer tokens, query secrets, and common credential keys) before printing
summaries, this isn't full proof so be careful with what you send.
