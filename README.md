# AVR-VM
[![Build Status](https://travis-ci.org/MackieLoeffel/avr-vm.svg?branch=master)](https://travis-ci.org/MackieLoeffel/avr-vm)

This is a VM for the AVR ATmega32 microcontroller written in Rust.
It can handle most of the instructions and has support for I/O, ADC,
timer and button interrupts. It also features a JIT compiler, which
compiles the AVR bytecode to x64 machinecode at runtime. It is quite
fast, about 5x faster than the real microcontroller.

This VM was built as part of an university course, so it is not under current
development. But if you want to use it and have a problem, feel free
to open an issue or a PR.

Since the code for the GUI was supplied by the professor, the code
is currently only working with the `no_gui` feature, see below.

Some test programs can be found in `./test`. The VM is only tested to
work with these programs. Only the instructions of these programs
are currently implemented (which are quite a few, but not all).

## Installation

For using this VM you need to do the following steps:

1.  clone this repository

2.  **ONLY FOR JIT-COMPILATION:**
    This project uses [Dynasm-rs](https://github.com/CensoredUsername/dynasm-rs) for the JIT-compiler, which is a
    compiler plugin and needs the nightly Rust compiler. A working
    version can be installed using [Rustup](https://rustup.rs/):
    `rustup override set nightly-2017-02-13`
    Furthermore the JIT-Compiler needs to be enabled using the feature "jit",
    see below.

3.  For running the tests and compiling C-Code and assembler to AVR
    bytecode, `gcc-avr` is needed. It can be installed either from
    <http://www.atmel.com/tools/ATMELAVRTOOLCHAINFORLINUX.aspx>
    or on Ubuntu/Debian/BashOnWindows using
    `sudo apt-get install gcc-avr`
    If you are using a different distribution, you can probably find
    an equivalent package there.

## Usage

You can execute the VM using the following command:
`cargo run --release -- ./test/jump/jump.bin`

All bytes, which are written to `UDR` by the microcontroller, are
displayed in the console.

You can exucute a different binary by changing `./test/jump/jump.bin`.

### Without GUI

The GUI can be disabled using
`cargo run --release --features no_gui -- ./test/jump/jump.bin`

### Use the JIT compiler

The JIT-Compiler can be enabled with the following flags:
`cargo run --release --features jit -- ./test/jump/jump.bin`

You need to make sure, that you are using a nightly version of the
Rust compiler, if you want to use the JIT-compiler. For
instructions, which version to use, see above.

## Material

-   [Instruction Set](http://www.atmel.com/images/Atmel-0856-AVR-Instruction-Set-Manual.pdf)
-   [Hardware Description](http://www.atmel.com/images/doc2503.pdf)
