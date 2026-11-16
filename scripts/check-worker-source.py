#!/usr/bin/env python3
"""Immutable worker checks; explicit daily mode records exact equivalent wrappers."""
import argparse
import json
import os
import subprocess
import tomllib
from pathlib import Path



def require_same_library(current, pinned):
    if current != pinned:
        raise SystemExit("Compatible Runtime wrapper changed the pinned library tree")


def main():
    root = Path(__file__).resolve().parents[1]
    source_lock = root / "worker/source.lock.json"
    original_lock = source_lock.read_text()
    pin = json.loads(original_lock)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--record-equivalent-wrapper", action="store_true")
    args = parser.parse_args()
    record = args.record_equivalent_wrapper
    if record and os.environ.get("YIJIE_DEVELOPMENT_CLIENT") != "true":
        raise SystemExit("Equivalent wrapper recording requires the fixed development preflight")
    codex = root.parent / "yijie-codex"
    head = subprocess.check_output(["git", "-C", str(codex), "rev-parse", "HEAD"], text=True).strip()
    # The daily preflight may record a new wrapper only after proving that its
    # complete library tree, working bytes and dependency closure remain exact.
    wrappers = pin.get("compatibleRuntimeWrapperCommits", [])
    unknown = head not in [pin["sourceCommit"], *wrappers]
    if unknown and not record:
        raise SystemExit("Pinned Codex library source commit differs; explicit source review required")
    for revision in [pin["sourceCommit"], *wrappers, *([head] if unknown else [])]:
        if len(revision) != 40 or any(c not in "0123456789abcdef" for c in revision):
            raise SystemExit("Library authority must use full Git commits")
        tree = subprocess.check_output(["git", "-C", str(codex), "rev-parse", revision + ":codex-rs"], text=True).strip()
        original = subprocess.check_output(["git", "-C", str(codex), "rev-parse", pin["sourceCommit"] + ":codex-rs"], text=True).strip()
        require_same_library(tree, original)
    subprocess.run(["git", "-C", str(codex), "diff", "--exit-code", pin["sourceCommit"], "--", "codex-rs"], check=True, stdout=subprocess.DEVNULL)
    if subprocess.check_output(["git", "-C", str(codex), "ls-files", "--others", "--exclude-standard", "--", "codex-rs"], text=True).strip():
        raise SystemExit("Untracked files in the pinned library source require explicit review")
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
    if unknown:
        if source_lock.read_text() != original_lock:
            raise SystemExit("Worker source lock changed during preflight")
        pin["compatibleRuntimeWrapperCommits"] = [*wrappers, head]
        temporary = source_lock.with_name(source_lock.name + "." + str(os.getpid()) + ".tmp")
        with temporary.open("x") as output:
            output.write(json.dumps(pin, indent=2) + "\n")
        temporary.replace(source_lock)
        print("Recorded equivalent Runtime wrapper after exact library and dependency verification")
    print("Fixed Codex source and explicitly pinned worker/schema validation dependencies verified; local candidate only")


if __name__ == "__main__":
    main()
