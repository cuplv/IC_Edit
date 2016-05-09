#!/bin/sh
# 10*1024*1024 = 10 MB
# export RUST_MIN_STACK=10485760
# cargo build --verbose
# cargo test --verbose -- --nocapture
# cargo bench --verbose

# ADAPTON_STRUCTURAL: Toggle this environment definition to include
# extra well-formedness checking (adds lots of extra overhead to
# Adapton, but catches naming errors early)
#
# export ADAPTON_STRUCTURAL=defined

# ADAPTON_WRITE_DCG: Toggle this environment definition to write, for
# each ADAPTON_CHECK_DCG check, the current DCG (as a graphviz graph
# in dot format) to the local filesystem (the current working
# directory).
#
# export ADAPTON_WRITE_DCG=defined

# ADAPTON_CHECK_DCG: Toggle this environment definition to
# periodically check the DCG for well-formedness.
# 
# export ADAPTON_CHECK_DCG=defined

# the switch '-n' means "no cursors"
cargo run -- windowless -s 0 -c 1000 -n

cargo run -- windowless -s 0 -c 1000
