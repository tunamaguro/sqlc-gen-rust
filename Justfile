_default:
  just --list 

set dotenv-filename := ".dev.env"
set dotenv-load

alias f:= format
alias l:= lint
alias lf:= lint-fix

setup-tools:
    rustup target add wasm32-wasip1

# format
format:
    cargo fmt --all

# format in CI
format-ci:
    RUSTFLAGS="--deny warnings" cargo fmt --all --check

# Show lint error
lint:
    cargo clippy --workspace --all-targets --all-features 

# Fix clippy error
lint-fix:
    cargo clippy --fix --workspace --all-targets --all-features --allow-dirty --allow-staged

# lint in CI
lint-ci:
    RUSTFLAGS="--deny warnings" cargo clippy --workspace --all-targets --all-features

# Run tests
test:
    cargo test --workspace

# rebuild plugin and generate sqlc
generate:
    #!/usr/bin/env bash
    set -euxo pipefail

    cargo build --target wasm32-wasip1

    WASM_SHA256=$(sha256sum target/wasm32-wasip1/debug/sqlc-gen-rust.wasm | awk '{print $1}');
    sed "s/\$WASM_SHA256/${WASM_SHA256}/g" sqlc.json > _sqlc_dev.json
    sqlc generate -f _sqlc_dev.json

    rm _sqlc_dev.json

build-release:
    #!/usr/bin/env bash
    set -euxo pipefail

    cargo build --target wasm32-wasip1 --release --locked
    WASM_SHA256=$(sha256sum target/wasm32-wasip1/release/sqlc-gen-rust.wasm | awk '{print $1}');
    echo ${WASM_SHA256}
    