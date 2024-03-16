default:
    cargo run --bin r_git help

scratch:
  cargo run --bin scratch

run +args:
    cargo run --bin r_git {{args}}

build:
    cargo build

test:
    cargo test -- --test-threads=1