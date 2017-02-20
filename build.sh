#!/usr/bin/env zsh

PROGRAM="jump-time"
PROGRAMDIR="jump"
# set default value
: ${PROGRAMDIR:=$PROGRAM}

# cargo build --color=never && \
# cargo build --color=never --features jit && \
# cargo build --color=never --features no_gui && \
# cargo build --color=never --features "no_gui jit" && \
cargo test --color=never --features "jit" -- --nocapture && \
    cargo run --color=never --release --features "no_gui jit" -- ./test/$PROGRAMDIR/$PROGRAM.bin
# cargo test -- --nocapture
