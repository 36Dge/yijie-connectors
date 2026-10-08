#!/usr/bin/env bash
set -euo pipefail
connector_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
python3 "$connector_root/scripts/check-worker-source.py"
node "$connector_root/../yijie-contracts/scripts/sync-market-broker.mjs" --consumer=connectors --check
node "$connector_root/../yijie-contracts/scripts/sync-market-provider.mjs" --consumer=connectors --check
export CARGO_TARGET_DIR="${YIJIE_WORKER_TARGET_DIR:-$connector_root/../yijie-codex/codex-rs/target}"
cargo build --offline --locked --manifest-path "$connector_root/worker/Cargo.toml" --features broker-qualification --bin market-broker-qualification
python3 - "$connector_root" "$CARGO_TARGET_DIR/debug/market-broker-qualification" <<'PY'
import hashlib,json,shutil,sys
from pathlib import Path
root,binary=map(Path,sys.argv[1:])
content=binary.read_bytes();digest=hashlib.sha256(content).hexdigest()
destination=root/'bin/market-broker-qualification'/digest/'market-broker-qualification'
destination.parent.mkdir(parents=True,exist_ok=True)
if destination.exists():
    if destination.read_bytes()!=content:raise SystemExit('Existing content-addressed qualification differs; refusing overwrite')
else:shutil.copy2(binary,destination)
sources=['worker/Cargo.toml','worker/Cargo.lock','worker/src/broker.rs','worker/src/daily.rs','worker/src/gateway.rs','worker/src/broker_generated.rs','worker/src/selection_generated.rs','worker/src/generated.rs','worker/src/provider_generated.rs','worker/src/providers.rs','worker/src/tushare_oauth.rs','worker/src/native_library.rs','worker/qualification/broker.rs','worker/source.lock.json']
manifest={'schemaVersion':1,'candidateOnly':True,'qualificationOnly':True,'binary':str(destination),'sha256':digest,'sizeBytes':len(content),'codexSource':json.loads((root/'worker/source.lock.json').read_text())['sourceCommit'],'externalCallsEnabled':False,'sources':{name:hashlib.sha256((root/name).read_bytes()).hexdigest() for name in sources}}
(root/'bin/market-broker-qualification/current.json').write_text(json.dumps(manifest,indent=2)+'\n')
print('Built dedicated local Broker qualification:',destination)
PY
