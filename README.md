# AVR-VM

This is a VM for the AVR ATmega32 microcontroller written in Rust.
It can handle most of the instructions and has support for I/O, ADC,
timer and button interrupts. It also features a JIT compiler, which
compiles the AVR bytecode to x64 machinecode at runtime. It is quite
fast, about 5x faster than the real microcontroller.

This VM was built as part of an university course, so it is not under current
development. But if you want to use it and have a problem, feel free
to open an issue or a PR.

Since the original vm used some code from my professor, for which I
don't have a license, the current version in this repository doesn't
compile.

## Installation

For using this VM you need to do the following steps:
1.  clone this repository
2.  This project uses [Dynasm-rs](https://github.com/CensoredUsername/dynasm-rs) for the JIT-compiler, which is a
    compiler plugin and needs the nightly Rust compiler. A working
    version can be installed using [Rustup](https://rustup.rs/):

    `rustup override set nightly-2017-02-13`
3.  For running the tests and compiling C-Code and assembler to AVR
    bytecode, `gcc-avr` is needed. It can be installed either from
    <http://www.atmel.com/tools/ATMELAVRTOOLCHAINFORLINUX.aspx>
    or on Ubuntu/Debian/BashOnWindows using
    `sudo apt-get install gcc-avr`
    If you are using a different distribution, you can probably find
    an equivalent package there.

## Usage

You can exuecte the VM using the following command:
`cargo run --release -- <path to the binary with avr code>`

All bytes, which are written to `UDR` by the microcontroller, are
displayed in the console.

### Without GUI

The GUI can be disabled using
`cargo run --release --features no_gui -- <path to the binary with avr code>`

### Without JIT

The JIT-Compiler can be disabled with the following flags and the
VM falls back to an emulating mode:
`cargo run --release --features interpret -- <path to the binary with avr code>`
