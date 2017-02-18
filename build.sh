#!/usr/bin/env zsh

PROGRAM="jump-time"
PROGRAMDIR="jump"
# set default value
: ${PROGRAMDIR:=$PROGRAM}

cargo test --color=never -- --nocapture && \
    cargo run --color=never --release --features no_gui -- ./test/$PROGRAMDIR/$PROGRAM.bin
# cargo test -- --nocapture
