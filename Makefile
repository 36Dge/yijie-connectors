.PHONY: dev test lint generate worker-build worker-test worker-lint worker-stdio-qualification worker-broker-qualification contract-check

dev:
	go run ./cmd/connector-gateway

test:
	go test -race -cover ./...

lint:
	@test -z "$$(gofmt -l $$(find cmd internal -type f -name '*.go'))" || (gofmt -l $$(find cmd internal -type f -name '*.go') && exit 1)
	go vet ./...

generate:
	node ../yijie-contracts/scripts/sync-market-connectors.mjs --consumer=connectors
	node ../yijie-contracts/scripts/sync-market-broker.mjs --consumer=connectors
	node ../yijie-contracts/scripts/sync-market-provider.mjs --consumer=connectors

contract-check:
	node ../yijie-contracts/scripts/sync-market-connectors.mjs --consumer=connectors --check
	node ../yijie-contracts/scripts/sync-market-broker.mjs --consumer=connectors --check
	node ../yijie-contracts/scripts/sync-market-provider.mjs --consumer=connectors --check

worker-build:
	bash scripts/build-market-worker.sh

worker-test: contract-check
	python3 scripts/check-worker-source.py
	CARGO_TARGET_DIR="$(abspath ../yijie-codex/codex-rs/target)" cargo test --offline --locked --manifest-path worker/Cargo.toml --bin yijie-mcp-worker

worker-lint: contract-check
	python3 scripts/check-worker-source.py
	rustfmt --edition 2024 --config skip_children=true --check worker/src/main.rs worker/src/native_library.rs worker/src/broker.rs worker/src/daily.rs worker/src/gateway.rs worker/src/providers.rs worker/src/generic.rs worker/src/google_calendar.rs worker/src/provider_registry.rs worker/src/oauth_policy.rs worker/src/credentials.rs worker/src/secret_guard.rs worker/src/tushare_oauth.rs worker/qualification/broker.rs
	CARGO_TARGET_DIR="$(abspath ../yijie-codex/codex-rs/target)" cargo clippy --offline --locked --manifest-path worker/Cargo.toml --no-deps -- -D warnings

worker-stdio-qualification:
	python3 scripts/check-worker-source.py
	CARGO_TARGET_DIR="$(abspath ../yijie-codex/codex-rs/target)" cargo test --offline --locked --manifest-path worker/Cargo.toml --features stdio-qualification --test stdio_bridge -- --nocapture

worker-broker-qualification: contract-check
	python3 scripts/check-worker-source.py
	CARGO_TARGET_DIR="$(abspath ../yijie-codex/codex-rs/target)" cargo test --offline --locked --manifest-path worker/Cargo.toml --features broker-qualification --bin yijie-mcp-worker
	bash scripts/build-market-broker-qualification.sh
