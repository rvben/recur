.PHONY: build test lint fmt-check fmt install clean release release-patch release-minor release-major

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

lint:
	cargo clippy -- -D warnings

fmt-check:
	cargo fmt -- --check

fmt:
	cargo fmt

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
