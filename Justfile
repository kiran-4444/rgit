default:
    cargo run --bin r_git help

scratch:
  cargo run --bin scratch

run arg:
    cargo run --bin r_git {{arg}}

build:
    cargo build --release

test:
    cargo test -- --test-threads=1