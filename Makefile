.PHONY: dev test lint generate

dev:
	go run ./cmd/connector-gateway

test:
	go test ./...

lint:
	go test ./...

generate:
	echo "No generated assets yet"
