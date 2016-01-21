#!/bin/sh
# 10*1024*1024 = 10 MB
# export RUST_MIN_STACK=10485760
# cargo build --verbose
# cargo test --verbose -- --nocapture
# cargo bench --verbose

# Toggle this line to include extra well-formedness checking
# (adds lots of extra overhead to Adapton, but catches naming errors early)
#
# export ADAPTON_STRUCTURAL=1

# Toggle this line to write, for each ADAPTON_CHECK_DCG check, the
# current DCG (as a graphviz graph in dot format) to the local
# filesystem (the current working directory).
#
# export ADAPTON_WRITE_DCG=1

# Toggle this line to periodically check the DCG for well-formedness
# export ADAPTON_CHECK_DCG=1

# the switch '-n' means "no cursors"
cargo run -- windowless -s 0 -c 10000 -n

# cargo run -- windowless -s 0 -c 300
