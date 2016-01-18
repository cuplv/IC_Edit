#!/bin/sh
# 10*1024*1024 = 10 MB
# export RUST_MIN_STACK=10485760
# cargo build --verbose
# cargo test --verbose -- --nocapture
# cargo bench --verbose

# Toggle this line to include extra well-formedness checking
# (adds lots of extra overhead to Adapton, but catches naming errors early)

export ADAPTON_CHECK_DCG=1

# -n means no cursors
cargo run -- windowless -s 0 -c 500 -n

cargo run -- windowless -s 0 -c 300
