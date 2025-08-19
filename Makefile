.PHONY: bootstrap up down logs perf build-node
bootstrap:
	npm -C services/explorer-api ci || true
up:
	docker compose up -d --build
down:
	docker compose down -v
logs:
	docker compose logs -f
build-node:
	cargo build --release -p qc-node
perf:
	gh workflow run perf || echo "Open Actions → perf"
