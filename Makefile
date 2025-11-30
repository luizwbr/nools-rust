.PHONY: help build test bench doc clean examples

help:
	@echo "Nools-RS Development Commands"
	@echo ""
	@echo "  make build      - Build the project in release mode"
	@echo "  make test       - Run all tests"
	@echo "  make bench      - Run benchmarks"
	@echo "  make doc        - Generate and open documentation"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make examples   - Run all examples"
	@echo "  make check      - Check code without building"
	@echo "  make fmt        - Format code"
	@echo "  make clippy     - Run clippy lints"
	@echo "  make all        - Format, check, test, and build"

build:
	cargo build --release

test:
	cargo test --all-features

bench:
	cargo bench

doc:
	cargo doc --no-deps --open

clean:
	cargo clean

examples:
	@echo "Running hello_world..."
	@cargo run --example hello_world
	@echo ""
	@echo "Running fibonacci..."
	@cargo run --example fibonacci
	@echo ""
	@echo "Running state_machine..."
	@cargo run --example state_machine

check:
	cargo check --all-features

fmt:
	cargo fmt

clippy:
	cargo clippy --all-features -- -D warnings

all: fmt check test build
	@echo "All checks passed!"

install-dev:
	rustup component add rustfmt clippy
	cargo install cargo-watch cargo-expand

watch:
	cargo watch -x check -x test

coverage:
	cargo tarpaulin --out Html --output-dir coverage
