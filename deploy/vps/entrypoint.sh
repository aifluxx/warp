#!/bin/sh
# Starts the headless Warp TUI.
#
# The container entrypoint runs as root only long enough to make the state
# directory writable, then drops to the unprivileged `warp` user.
# Subcommands: serve (default), version, help, tui.

set -eu

STATE_DIR=/var/lib/warp-tui
BINARY=/opt/warp-oss/warp-tui-oss

mkdir -p \
  "$STATE_DIR/home" \
  "$STATE_DIR/config" \
  "$STATE_DIR/data" \
  "$STATE_DIR/state" \
  "$STATE_DIR/cache"
chown -R warp:warp "$STATE_DIR"

# An empty value from Compose is not the same as an unset variable. Clap would
# treat an empty WARP_API_KEY as a provided key.
if [ -z "${WARP_API_KEY:-}" ]; then
  unset WARP_API_KEY
fi
if [ -z "${TTYD_USERNAME:-}" ]; then
  unset TTYD_USERNAME
fi
if [ -z "${TTYD_PASSWORD:-}" ]; then
  unset TTYD_PASSWORD
fi

export HOME="$STATE_DIR/home"
export USER=warp
export LOGNAME=warp
export XDG_CONFIG_HOME="$STATE_DIR/config"
export XDG_DATA_HOME="$STATE_DIR/data"
export XDG_STATE_HOME="$STATE_DIR/state"
export XDG_CACHE_HOME="$STATE_DIR/cache"
export LANG="${LANG:-C.UTF-8}"
export TERM="${TERM:-xterm-256color}"

cd "$HOME"

run_as_warp() {
  exec runuser -u warp --preserve-environment -- "$@"
}

case "${1:-serve}" in
  version)
    run_as_warp "$BINARY" --version
    ;;
  help)
    run_as_warp "$BINARY" --help
    ;;
  tui)
    shift
    run_as_warp "$BINARY" "$@"
    ;;
  serve)
    # One line in the container log, with no environment dump.
    runuser -u warp --preserve-environment -- "$BINARY" --version
    port="${TTYD_PORT:-7681}"
    if [ -n "${TTYD_USERNAME:-}" ] && [ -n "${TTYD_PASSWORD:-}" ]; then
      exec runuser -u warp --preserve-environment -- \
        ttyd -W -m 2 -i 0.0.0.0 -p "$port" \
        -c "${TTYD_USERNAME}:${TTYD_PASSWORD}" \
        "$BINARY"
    fi
    exec runuser -u warp --preserve-environment -- \
      ttyd -W -m 2 -i 0.0.0.0 -p "$port" \
      "$BINARY"
    ;;
  *)
    run_as_warp "$@"
    ;;
esac
