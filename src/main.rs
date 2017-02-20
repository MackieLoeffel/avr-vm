#![cfg_attr(feature = "jit", feature(plugin))]
#![cfg_attr(feature = "jit", feature(abi_sysv64))]
#![cfg_attr(feature = "jit", plugin(dynasm))]
#![cfg_attr(feature = "strict", deny(warnings))]

#[cfg(feature = "jit")]
extern crate dynasmrt;
extern crate rand;

use std::env::args;
#[macro_use]
mod util;
mod decoder;
mod data;
mod cpu;
mod memory;
mod gui;
mod io;
mod widgets;
mod ports;
mod interrupts;
use std::ffi::OsString;
use cpu::{Cpu};
use memory::{Memory};
use io::IO;
use widgets::{Widget};
use widgets::WidgetType::*;

fn main() {
    if args().count() != 2 {
        println!("usage: vm <program>");
        return;
    }

    #[cfg(feature = "no_gui")]
    {
        let mem = Memory::new(OsString::from(args().nth(1).expect("There must be an argument")), None);
        let mut cpu = Cpu::new(mem, true);
        while cpu.step() {}
        // if statement to remove unused code warning for gui code
        if 1 == 1 { return; }
    }

    let io = IO::new();
    gui::init();
    let widgets = [
        Widget::new("red0", Led(0xff, 0x00, 0x00, &io.vcc, &io.p[3][7])),
        Widget::new("green0", Led(0x00, 0xff, 0x00, &io.vcc, &io.p[2][0])),
        Widget::new("yellow0", Led(0xff, 0xff, 0x00, &io.vcc, &io.p[2][1])),
        Widget::new("blue0", Led(0x00, 0x00, 0xff, &io.vcc, &io.p[2][6])),
        Widget::new("red1", Led(0xff, 0x00, 0x00, &io.vcc, &io.p[2][7])),
        Widget::new("green1", Led(0x00, 0xff, 0x00, &io.vcc, &io.p[0][7])),
        Widget::new("yellow1", Led(0xff, 0xff, 0x00, &io.vcc, &io.p[0][6])),
        Widget::new("blue1", Led(0x00, 0x00, 0xff, &io.vcc, &io.p[0][5])),
        Widget::new("button0", Button(&io.p[3][3])),
        Widget::new("button1", Button(&io.p[3][2])),
        Widget::new("potentiometer", Poti(&io.gnd, &io.p[0][1], &io.vcc)),
        Widget::new("light sensor", Poti(&io.gnd, &io.p[0][0], &io.vcc)),
        Widget::new("dis2", Seg7(&io.p[1][1], &io.p[1][0], &io.p[3][1], &io.p[1][6], &io.vcc,
                                 &io.p[1][5], &io.p[1][4], &io.p[3][1], &io.p[1][3], &io.p[1][2])),
        Widget::new("dis1", Seg7(&io.p[1][1], &io.p[1][0], &io.p[3][0], &io.p[1][6], &io.vcc,
                                 &io.p[1][5], &io.p[1][4], &io.p[3][0], &io.p[1][3], &io.p[1][2]))
    ];

    io.nreset.set_ext(io::LOGIC_0);
    io.gnd.set_ext(io::LOGIC_0);
    io.vcc.set_ext(io::LOGIC_1);

    let mem = Memory::new(OsString::from(args().nth(1).expect("There must be an argument")), Some(&io));
    let mut cpu = Cpu::new(mem, false);

    while gui::step() {
        for _ in 0..100 {
            cpu.step();
        }
        for widget in widgets.iter() {
            widget.step();
        }
    }
    return;
}
