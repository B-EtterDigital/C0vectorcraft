#!/bin/sh
# Run this whole sequence inside one admitted slot; Cargo never exceeds two jobs.
set -eu
cd "$(dirname "$0")/../.."
export CARGO_BUILD_JOBS=2
export BINARYEN_CORES="$CARGO_BUILD_JOBS"
export RUST_TEST_THREADS="$CARGO_BUILD_JOBS"
if [ -n "${CRAFT_FONTS_DIR:-}" ] && [ ! -d "$CRAFT_FONTS_DIR" ]; then
    echo "CRAFT_FONTS_DIR is missing; using built-in fallback fonts." >&2
    unset CRAFT_FONTS_DIR
fi
if [ -n "${CRAFT_FONTS_DIR:-}" ]; then
    case "$CRAFT_FONTS_DIR" in /*) ;; *) export CRAFT_FONTS_DIR="$PWD/$CRAFT_FONTS_DIR" ;; esac
fi
log_dir="${C0VECTOR_LOG_DIR:-/tmp/c0vector-gates}"
mkdir -p "$log_dir"
log_dir="$(cd "$log_dir" && pwd)"
stage() {
    stage_name="$1"
    shift
    echo "Starting $stage_name"
    if "$@" > "$log_dir/$stage_name.log" 2>&1; then
        echo "PASS $stage_name"
    else
        stage_rc=$?
        echo "FAIL $stage_name ($stage_rc): $log_dir/$stage_name.log" >&2
        exit "$stage_rc"
    fi
}
stage fmt cargo fmt --all
stage native-check cargo check -p vectorcraft-ui-egui -p vectorcraft-web --features page-control
stage host-tests cargo test --no-fail-fast -p vectorcraft-ui-egui --test c0_host_contract --test c0_chrome --test c0_theme --test c0_untrusted
stage web-tests cargo test -p vectorcraft-web --features page-control
stage wasm-feature cargo check --target wasm32-unknown-unknown -p vectorcraft-web --features page-control
stage clippy-feature cargo clippy -p vectorcraft-ui-egui -p vectorcraft-web -p vectorcraft-doc --all-targets --features page-control -- -D warnings
stage clippy-wasm cargo clippy --target wasm32-unknown-unknown -p vectorcraft-web --features page-control -- -D warnings
stage ci cargo xtask ci
cat /proc/loadavg > "$log_dir/load-before-build.txt"
cd c0/vectorcraft-host
stage page-build env NO_COLOR=true trunk build --release
cat /proc/loadavg > "$log_dir/load-after-build.txt"
cd ../..
# A local driver stays outside the Rust-only repository. Run it under this same admission.
smoke_script="${C0VECTOR_SMOKE_SCRIPT:-$log_dir/smoke.cjs}"
if [ -f "$smoke_script" ]; then
    python3 -u -m http.server 0 --bind 127.0.0.1 --directory dist/c0host > "$log_dir/http.log" 2>&1 &
    server_pid=$!
    trap 'kill "$server_pid" 2>/dev/null || true' EXIT HUP INT TERM
    port=""
    attempt=0
    while [ -z "$port" ] && [ "$attempt" -lt 50 ]; do
        port="$(sed -n 's/.* port \([0-9][0-9]*\) .*/\1/p' "$log_dir/http.log")"
        attempt=$((attempt + 1))
        [ -n "$port" ] || sleep 0.1
    done
    [ -n "$port" ] || { echo "Smoke HTTP server did not start" >&2; exit 1; }
    stage browser-smoke env C0VECTOR_SMOKE_URL="http://127.0.0.1:$port" C0VECTOR_SMOKE_SCREENSHOT="$log_dir/smoke.png" node "$smoke_script"
else
    echo "SKIP browser-smoke: set C0VECTOR_SMOKE_SCRIPT to a local Playwright driver"
fi
