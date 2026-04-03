.PHONY: build test lint fmt check install clean release release-patch release-minor release-major

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

lint:
	cargo fmt -- --check
	cargo clippy -- -D warnings

fmt:
	cargo fmt

check: lint test

install:
	cargo install --path .

clean:
	cargo clean

release-patch:
	vership bump patch

release-minor:
	vership bump minor

release-major:
	vership bump major
