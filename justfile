default:
    just --list

build:
    cargo build --release

install:
    cargo install --path . --locked --force

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
