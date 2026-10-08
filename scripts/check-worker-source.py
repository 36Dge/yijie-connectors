#!/usr/bin/env python3
"""Read-only immutable library/dependency checks for the local worker build."""
import json
import subprocess
import tomllib
from pathlib import Path

root = Path(__file__).resolve().parents[1]
pin = json.loads((root / "worker/source.lock.json").read_text())
codex = root.parent / "yijie-codex"
head = subprocess.check_output(["git", "-C", str(codex), "rev-parse", "HEAD"], text=True).strip()
if head != pin["sourceCommit"]:
    raise SystemExit("Pinned Codex library source commit differs; explicit source review required")
subprocess.run(["git", "-C", str(codex), "diff", "--exit-code", pin["sourceCommit"], "--", "codex-rs"], check=True, stdout=subprocess.DEVNULL)
if subprocess.check_output(["rustc", "--version"], text=True).split()[1] != pin["rustVersion"]:
    raise SystemExit("Pinned Rust version differs")
reference = tomllib.loads((codex / "codex-rs/Cargo.lock").read_text())["package"]
actual = tomllib.loads((root / "worker/Cargo.lock").read_text())["package"]
expected_external = {(p["name"], p["version"], p.get("source"), p.get("checksum")) for p in reference if "source" in p}
schema_pin = json.loads((root / "worker/schema-validation.lock.json").read_text())
if schema_pin["root"] != "jsonschema" or schema_pin["version"] != "0.58.6" or schema_pin["defaultFeatures"]:
    raise SystemExit("Schema validator dependency policy differs")
expected_schema = {(p["name"], p["version"], p["source"], p["checksum"]) for p in schema_pin["packages"]}
manifest = tomllib.loads((root / "worker/Cargo.toml").read_text())
if manifest["dependencies"].get("jsonschema") != {"version": "=0.58.6", "default-features": False}:
    raise SystemExit("Schema validator must remain pinned with file/network resolution disabled")
actual_extra = {(p["name"], p["version"], p.get("source"), p.get("checksum")) for p in actual if "source" in p} - expected_external
if actual_extra != expected_schema:
    raise SystemExit("Schema validator dependency closure differs from its reviewed lock")
for package in actual:
    if "source" in package and (package["name"], package["version"], package.get("source"), package.get("checksum")) not in expected_external | expected_schema:
        raise SystemExit("Worker external dependency differs from fixed Codex lock: " + package["name"])
print("Fixed Codex source and explicitly pinned worker/schema validation dependencies verified; local candidate only")
