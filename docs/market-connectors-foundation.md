# FEAT-157 market connector foundation

This local implementation provides the safe 51-entry product catalog and a
status-only native worker. It does not activate, connect, authorize or execute any
external provider. All 51 provider qualifications remain incomplete.

## Authority and consumers

`catalog/market-catalog.v1.json` is the Connectors-owned product source consumed by
Desktop packaging. Its entries use the generated Contracts `CatalogEntry`; the
asset envelope contains only `catalogRevision` and `catalog`. It includes no
endpoint, command, OAuth URL, environment/header name or secret. `catalog.Read`
validates the exact 51 identifiers and generated types before packaging.

`catalog/provider-candidates.v1.json` preserves non-executable provider reference
material separately. Every entry has `activationAllowed=false`, empty accepted
targets and no approved tools. Neither the public catalog's auth label nor an
input reference URL grants activation. Source provenance records the audited
input digest; future edits are owned and reviewed in this repository.

Installation, desired enablement, non-secret settings and opaque credential
references are Native SQLCipher product state, subject to the reviewed ADR-0013
scope extension. Connectors does not create another installation database. Host
keeps adaptation/ability mappings only. There is no new public HTTP API in this
foundation: `internal/app.MarketFoundation` is an internal assembly point and
must later be connected through the approved identity/capability contract.

## Fixed Codex library worker

`worker/source.lock.json` pins the Codex source commit and Rust compiler. The
Cargo path dependencies use that source without copying or changing it;
`scripts/check-worker-source.py` rejects changed tracked `codex-rs` sources,
incorrect commits/toolchains and any external lock dependency absent from the
fixed Codex lock. Workspace-local Codex packages resolve at their canonical
0.144.6 version; the worker has its own generated `Cargo.lock`.

`native_library.rs` uses the actual Codex OAuth storage enum and public HTTP/OAuth
entrypoints. Store policy is strictly Keyring, with stable scope/connection
aliases distinct from normal Codex names. The IO adapters currently require an
unconstructible private approved target: no status protocol input can supply an
endpoint or enable these methods. This is a compile-checked dependency boundary,
not a working external OAuth integration or tested vault.

The only private JSONL method is Contracts `auth_status`. Its request is closed;
known services return `not_qualified`, `unknown` authorization, `disconnected`
and `executionAvailable=false`. Unknown service IDs have a distinct stable error.
Status never reads Keychain, starts a browser, calls HTTP, launches an MCP process
or accepts credentials. No arbitrary protocol proxy is exposed.

The worker accepts bounded 16 KiB request lines and exits normally on stdin EOF.
The Go owner validates a canonical build manifest and binary digest, serializes
requests, sends EOF after each status query and drains bounded output. Timeout
retains the process owner as STOP_PENDING; there is no kill, signal or
CommandContext termination fallback. A later query cannot start another child
while cleanup remains pending. The real-worker integration test uses only the
normally built binary, never a fixture executable.

The 2026-10-07 follow-up audit corrected owner admission: cancelled requests stop
before artifact access, queue waiting observes cancellation, and cancellation is
rechecked after artifact validation before process start. `Close` permanently
closes admission before awaiting normal exit; it is idempotent and cannot revive
the owner after a timeout. `AwaitNormalExit` remains a wait-only operation. The
exit notification is broadcast so a query and a simultaneous close can both
observe the same normally reaped child. Timeout still retains the owner as
STOP_PENDING. These private lifecycle changes do not change the generated wire
contract or activate providers. Results are in
`docs/market-owner-lifecycle-verification.json`.

## Build and validation

```sh
make generate
make contract-check
make worker-test
make worker-lint
make worker-build
YIJIE_MARKET_WORKER_TEST_MANIFEST="$PWD/bin/market-worker/current.json" go test -race ./...
```

The build reuses the existing Cargo target cache and writes only the new worker
binary. Runtime binaries/manifests are not rebuilt or overwritten. A copied
worker artifact is stored under its content digest in this project's ignored
`bin/market-worker/`; a different existing content-addressed binary is rejected.
All Cargo actions are offline/locked. These are local candidate paths and hashes,
not release pins or signing/production approval.

## Next integration boundaries

Before enabling HTTP/OAuth, implement accepted provider evidence, exact origins,
callback/scope/credential ownership, cancellation and Keyring qualification;
then add versioned control operations through Contracts. Runtime should only
receive local restricted Gateway abilities, never platform tokens. Before tool
execution, connect Host native-thread/turn/revision binding and existing policy
approval authority, with Gateway checking them again at actual execution.

Google stdio services remain blocked pending package and safe EOF byte-bridge
qualification. Do not call Codex's current LocalStdioServerLauncher because it
has kill-on-drop/forced termination behavior. Google Calendar's own credential
and token files require separate safe storage qualification; the Codex Keyring
setting does not protect them. No real accounts, paid/provider calls, external
write operations, attack fixtures or forced process tests are authorized by this
foundation's checks.

## Completed local stdio byte-bridge qualification

`make worker-stdio-qualification` explicitly enables a Cargo integration test and
a dedicated project-built `stdio-qualification-server`; this is not part of the
product worker's control protocol or its normal build. It does not impersonate or
replace an installed executable. The test calls the fixed public
`InProcessTransportFactory` and passes its `DuplexStream` through ordinary byte
copy tasks to a separately owned child. The standard Codex stdio launcher is never
used. One initialize/list/lookup succeeds, then the owner normally closes stdin,
waits for child exit0 and joins both bridge tasks; no termination signal or
kill-on-drop fallback exists. The checked source/dependency closure is unchanged.

This proves the public byte interface can support a normal EOF owner without a
Runtime patch. It does not qualify the two Google packages, descendants, tokens,
recovery or a production stdio bridge. Exact local result is recorded in the
FEAT-157 runtime-qualification/stdio-run-01.json evidence in the metadata repo.
