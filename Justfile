default:
    cargo run --bin rgit help

scratch:
  cargo run --bin scratch

run +args:
    cargo run --bin rgit {{args}}

build:
    cargo build

test:
    cargo test -- --test-threads=1