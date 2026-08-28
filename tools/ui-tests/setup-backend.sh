#!/usr/bin/env bash
# Starts (or reuses) the Obsidian Server Panel backend for the UI test suite.
# Exclusive ports: backend 18104, vite 3174 (Agent C).
set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PORT=18104
VITE_PORT=3174
LOG="$ROOT/target/ui-tests/backend.log"
PIDFILE="$ROOT/target/ui-tests/backend.pid"

is_healthy() {
    curl -s -m 3 "http://127.0.0.1:${PORT}/api/info" | grep -q '"version"'
}

# Reuse an existing healthy backend; kill anything else on the port.
if is_healthy; then
    echo "[setup-backend] backend already listening on ${PORT} and healthy - reusing"
    exit 0
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

mkdir -p "$ROOT/target/ui-tests"
# vite-actix locates the vite binary via `which vite`; the worktree's node_modules is a
# symlink whose .bin must be on PATH (pnpm exec is deliberately bypassed: it tries to
# reinstall/purge the shared node_modules symlink).
export PATH="$ROOT/node_modules/.bin:$PATH"
export OBSIDIAN_VITE_PORT="$VITE_PORT"

# Anchor the backend's working directory at the worktree root: the debug binary resolves
# its dev-env as "./target/dev-env" RELATIVE TO ITS STARTUP CWD, and run-all invokes this
# script with cwd=tools/ui-tests - without the cd the backend silently uses
# tools/ui-tests/target/dev-env while the harness seeds fixtures into target/dev-env.
cd "$ROOT"
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
