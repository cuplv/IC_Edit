#!/bin/sh
# 10*1024*1024 = 10 MB
# export RUST_MIN_STACK=10485760
# cargo build --verbose
# cargo test --verbose -- --nocapture
# cargo bench --verbose

# -n no cursors
cargo run -- windowless -s 0 -c 1000 -n
