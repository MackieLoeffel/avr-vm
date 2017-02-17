#!/usr/bin/env zsh

PROGRAM="boardtest"
PROGRAMDIR=
# set default value
: ${PROGRAMDIR:=$PROGRAM}

cargo test --color=never -- --nocapture && \
    cargo run --color=never --release -- ./vm-atmega32/test/$PROGRAMDIR/$PROGRAM.bin
# cargo test -- --nocapture
