default:
    just --list

build:
    cargo build --release

format:
    cargo fmt

lint:
    cargo clippy --all-targets

unit-test:
    cargo test

test: unit-test

check:
    cargo fmt -- --check
    cargo clippy --all-targets
    cargo test
    cargo build --release
