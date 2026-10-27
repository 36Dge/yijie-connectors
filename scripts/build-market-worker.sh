#!/usr/bin/env bash
set -euo pipefail
connector_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
output_args=()
if [[ "$#" != "0" ]]; then
  [[ "$#" == "2" && "$1" == "--output-root" ]] || { echo "Expected only --output-root <new generation directory>" >&2; exit 1; }
  python3 -B "$connector_root/scripts/worker_build_output.py" check --output-root "$2"
  output_args=(--output-root "$2")
fi
python3 "$connector_root/scripts/check-worker-source.py"
node "$connector_root/../yijie-contracts/scripts/sync-market-connectors.mjs" --consumer=connectors --check
node "$connector_root/../yijie-contracts/scripts/sync-market-broker.mjs" --consumer=connectors --check
node "$connector_root/../yijie-contracts/scripts/sync-market-provider.mjs" --consumer=connectors --check
export CARGO_TARGET_DIR="${YIJIE_WORKER_TARGET_DIR:-$connector_root/../yijie-codex/codex-rs/target}"
cargo build --offline --locked --manifest-path "$connector_root/worker/Cargo.toml" --bin yijie-mcp-worker
python3 -B "$connector_root/scripts/worker_build_output.py" publish --binary "$CARGO_TARGET_DIR/debug/yijie-mcp-worker" "${output_args[@]+"${output_args[@]}"}"
