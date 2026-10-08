#!/usr/bin/env bash
set -euo pipefail
connector_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
python3 "$connector_root/scripts/check-worker-source.py"
node "$connector_root/../yijie-contracts/scripts/sync-market-connectors.mjs" --consumer=connectors --check
node "$connector_root/../yijie-contracts/scripts/sync-market-broker.mjs" --consumer=connectors --check
node "$connector_root/../yijie-contracts/scripts/sync-market-provider.mjs" --consumer=connectors --check
export CARGO_TARGET_DIR="${YIJIE_WORKER_TARGET_DIR:-$connector_root/../yijie-codex/codex-rs/target}"
cargo build --offline --locked --manifest-path "$connector_root/worker/Cargo.toml" --bin yijie-mcp-worker
python3 - "$connector_root" "$CARGO_TARGET_DIR/debug/yijie-mcp-worker" <<'PY'
import hashlib,json,shutil,sys
from pathlib import Path
root,binary=map(Path,sys.argv[1:])
content=binary.read_bytes();digest=hashlib.sha256(content).hexdigest()
destination=root/'bin/market-worker'/digest/'yijie-mcp-worker'
destination.parent.mkdir(parents=True,exist_ok=True)
if destination.exists():
    if destination.read_bytes()!=content:raise SystemExit('Existing content-addressed worker differs; refusing overwrite')
else:shutil.copy2(binary,destination)
manifest={'schemaVersion':2,'candidateOnly':True,'binary':str(destination),'sha256':digest,'sizeBytes':len(content),'codexSource':json.loads((root/'worker/source.lock.json').read_text())['sourceCommit'],'externalCallsEnabled':True,'qualificationOnly':False,'providerProfile':'generic-mcp-v1'}
(root/'bin/market-worker/current.json').write_text(json.dumps(manifest,indent=2)+'\n')
print('Built local status/Broker worker:',destination)
PY
