"""Private canonical worker artifact output; no Runtime or provider operations."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat


def validate_output(value):
    root = Path(value)
    if not root.is_absolute() or str(root) != value or root.resolve(strict=True) != root:
        raise ValueError("worker output must be a canonical absolute directory")
    s = root.lstat()
    if not stat.S_ISDIR(s.st_mode) or s.st_uid != os.geteuid() or s.st_mode & 0o022:
        raise ValueError("worker output ownership/protection")
    if os.path.lexists(root / "current.json"):
        raise ValueError("worker generation already contains a manifest")
    return root


def publish(root, binary, output=None):
    target = validate_output(str(output)) if output is not None else root / "bin/market-worker"
    content = binary.read_bytes()
    digest = hashlib.sha256(content).hexdigest()
    destination = target / digest / "yijie-mcp-worker"
    destination.parent.mkdir(parents=True, exist_ok=True)
    if os.path.lexists(destination):
        if destination.is_symlink() or not destination.is_file() or destination.read_bytes() != content:
            raise ValueError("Existing content-addressed worker differs; refusing overwrite")
    else:
        # The bytes come only from the standard Cargo build above, never a fixture executable.
        with binary.open("rb") as src, destination.open("xb") as dst:
            shutil.copyfileobj(src, dst)
        shutil.copystat(binary, destination)
    manifest = {"schemaVersion": 2, "candidateOnly": True, "binary": str(destination), "sha256": digest,
                "sizeBytes": len(content), "codexSource": json.loads((root / "worker/source.lock.json").read_text())["sourceCommit"],
                "externalCallsEnabled": True, "qualificationOnly": False, "providerProfile": "generic-mcp-v1"}
    with (target / "current.json").open("x" if output is not None else "w") as stream:
        stream.write(json.dumps(manifest, indent=2) + "\n")
    return destination


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("action", choices=["check", "publish"])
    p.add_argument("--output-root")
    p.add_argument("--binary", type=Path)
    args = p.parse_args()
    if args.action == "check":
        if args.output_root is None or args.binary is not None:
            p.error("check requires only an output root")
        validate_output(args.output_root)
    else:
        if args.binary is None:
            p.error("publish requires the compiled binary")
        print("Built local status/Broker worker:", publish(Path(__file__).resolve().parents[1], args.binary,
                                                          Path(args.output_root) if args.output_root else None))


if __name__ == "__main__":
    main()
