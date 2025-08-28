.PHONY: build test run fmt clippy clean

build:
	cargo build --release

test:
	cargo test --all --release

run:
	cargo run -p qc-node --release

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all -- -D warnings

clean:
	cargo clean

dev:
	cargo run -p qc-node

check:
	cargo check --all

install:
	cargo install --path crates/node

doc:
	cargo doc --open
