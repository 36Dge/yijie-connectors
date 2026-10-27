# Local startup source review — 2026-10-23

`contract-impact = none`: this updates only source provenance for the existing
local worker. Its compiled library sources, dependencies, protocol, permissions
and execution behavior are unchanged. It is not a release qualification.

The worker previously pinned Codex commit
`7fd463bcef07f37b0211acd9f62b9f93ea0a4b12`; the current clean checkout is
`9154f27d0cdb526a5f5ba0913fe3e2448014b4a1`.
Both commits have the exact same `codex-rs` tree:
`755f32c45a4c30e805e3c7fe1db89dcb4dcc6e41`.
`git diff --exit-code 7fd463bcef07f37b0211acd9f62b9f93ea0a4b12 -- codex-rs`
also passes against the current working tree. All four worker Cargo path
dependencies remain inside that unchanged tree.

The seven changed paths are outside `codex-rs`: the optional web-search patch,
its isolated Runtime build/schema/manifest scripts and documentation. The worker
does not apply those patches. Its exact Rust version, Cargo dependency closure,
schema-validator restrictions and source checks remain enforced by
`scripts/check-worker-source.py`. Only `worker/source.lock.json`'s complete source
commit is updated; the canonical build creates the worker artifact.

The Host's private worker-artifact admission in
`internal/marketbroker/artifact.go` must pin that same full commit. Its equality
check is updated to the reviewed identity; all manifest, profile, binary hash,
size and ownership checks remain in place. Desktop's local Host snapshot is
refrozen through its canonical generator after consumer verification. Runtime
and catalog consumer metadata are also regenerated to reflect the already
committed sources; their code, schemas and catalog bytes do not change.
