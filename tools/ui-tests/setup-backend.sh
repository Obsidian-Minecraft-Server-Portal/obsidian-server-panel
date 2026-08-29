#!/usr/bin/env bash
# Starts (or reuses) the Obsidian Server Panel backend for the UI test suite.
# Default ports: backend 18104, vite 3174 - override with UI_TEST_PORT / UI_TEST_VITE_PORT
# for per-agent exclusivity (run-all.ts forwards the environment).
set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# Ports default to the shared suite assignment; override via UI_TEST_PORT/UI_TEST_VITE_PORT
# (or UI_BASE_URL/OBSIDIAN_VITE_PORT) when running against a private backend instance.
PORT="${UI_TEST_PORT:-18104}"
if [ -n "${UI_BASE_URL:-}" ]; then
    PORT="${UI_BASE_URL#*://}"   # strip scheme
    PORT="${PORT%%/*}"           # strip path
    PORT="${PORT##*:}"           # keep the port
fi
VITE_PORT="${UI_TEST_VITE_PORT:-${OBSIDIAN_VITE_PORT:-3174}}"
LOG="$ROOT/target/ui-tests/backend.log"
PIDFILE="$ROOT/target/ui-tests/backend.pid"
# The suite owns an isolated dev-env so manual testing data (real admin account,
# user-created servers, settings) can never interfere - and vice versa. The harness
# (tools/ui-tests/harness.ts) derives the same path for its fixtures.
STATE_ROOT="$ROOT/target/ui-tests/dev-env-root"

is_healthy() {
    curl -s -m 3 "http://127.0.0.1:${PORT}/api/info" | grep -q '"version"'
}

# A healthy backend whose working directory is NOT the suite state root would silently
# use the wrong dev-env; kill and relaunch it against the suite state.
if is_healthy; then
    CWD_OF="$(readlink "/proc/$(cat "$PIDFILE" 2>/dev/null)/cwd" 2>/dev/null || true)"
    if [ "$CWD_OF" = "$STATE_ROOT" ]; then
        echo "[setup-backend] backend already listening on ${PORT} and healthy - reusing"
        exit 0
    fi
    echo "[setup-backend] backend on ${PORT} uses cwd ${CWD_OF:-unknown} - restarting against the suite state root"
fi

if [ -f "$PIDFILE" ]; then
    OLD_PID="$(cat "$PIDFILE" 2>/dev/null || true)"
    if [ -n "$OLD_PID" ] && kill -0 "$OLD_PID" 2>/dev/null; then
        echo "[setup-backend] killing stale backend pid ${OLD_PID}"
        kill "$OLD_PID" 2>/dev/null || true
        sleep 1
    fi
fi
fuser -k "${PORT}/tcp" 2>/dev/null || true
# the backend spawns vite as a child; a killed backend can leave it holding the vite port
fuser -k "${VITE_PORT}/tcp" 2>/dev/null || true
sleep 1

mkdir -p "$STATE_ROOT" "$ROOT/target/ui-tests"
# vite-actix locates the vite binary via `which vite`; the worktree's node_modules is a
# symlink whose .bin must be on PATH (pnpm exec is deliberately bypassed: it tries to
# reinstall/purge the shared node_modules symlink).
export PATH="$ROOT/node_modules/.bin:$PATH"
export OBSIDIAN_VITE_PORT="$VITE_PORT"

# Run the backend with its working directory anchored at the suite state root: the debug
# binary resolves its dev-env as "./target/dev-env" RELATIVE TO ITS STARTUP CWD. Cargo
# still builds against the workspace manifest, so target/ artifacts stay shared.
cd "$STATE_ROOT"
nohup "$HOME/.cargo/bin/cargo" run --no-default-features --features sqlite \
    --manifest-path "$ROOT/Cargo.toml" -- --port "$PORT" > "$LOG" 2>&1 &
echo $! > "$PIDFILE"
echo "[setup-backend] started backend pid $(cat "$PIDFILE") (log: $LOG)"

for i in $(seq 1 240); do
    if is_healthy; then
        echo "[setup-backend] backend ready on http://127.0.0.1:${PORT} (vite ${VITE_PORT})"
        exit 0
    fi
    if ! kill -0 "$(cat "$PIDFILE")" 2>/dev/null; then
        echo "[setup-backend] backend process died - last log lines:" >&2
        tail -30 "$LOG" >&2
        exit 1
    fi
    sleep 2
done

echo "[setup-backend] backend did not become ready in time - last log lines:" >&2
tail -30 "$LOG" >&2
exit 1
