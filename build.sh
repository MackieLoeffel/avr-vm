#!/usr/bin/env zsh

PROGRAM="boardtest"
PROGRAMDIR=
# set default value
: ${PROGRAMDIR:=$PROGRAM}

# cargo build --color=never && \
# cargo build --color=never --features jit && \
# cargo build --color=never --no-default-features && \
# cargo build --color=never --no-default-features --features "jit" && \
# cargo test --color=never --features "jit" -- --nocapture && \
# cargo run --color=never --release --no-default-features --features "jit" -- ./test/$PROGRAMDIR/$PROGRAM.bin
cargo run --color=never --release --features "jit" -- ./test/$PROGRAMDIR/$PROGRAM.bin
# cargo test -- --nocapture
