#!/usr/bin/env bash
set -euo pipefail

# brb writes one JSON event to stdin for custom channels.
payload="$(cat)"

if [ -z "${payload}" ]; then
  echo "no payload received on stdin" >&2
  exit 1
fi

# Where to store raw payload logs.
log_file="${BRB_CUSTOM_LOG_FILE:-/tmp/brb-custom-channel.log}"
printf '%s\n' "${payload}" >> "${log_file}"

# Optional readable summary (requires jq).
if command -v jq >/dev/null 2>&1; then
  status="$(printf '%s' "${payload}" | jq -r '.status // "unknown"')"
  command_text="$(printf '%s' "${payload}" | jq -r '.command // [] | join(" ")')"
  exit_code="$(printf '%s' "${payload}" | jq -r '.exit_code // -1')"

  echo "brb custom notifier: ${status} (exit ${exit_code}) :: ${command_text}" >&2
fi

exit 0
