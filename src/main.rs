#![cfg_attr(feature = "jit", feature(plugin))]
#![cfg_attr(feature = "jit", feature(abi_sysv64))]
#![cfg_attr(feature = "jit", plugin(dynasm))]
#![cfg_attr(feature = "strict", deny(warnings))]

#[cfg(feature = "jit")]
extern crate dynasmrt;
extern crate rand;
#[cfg(feature = "gui")]
extern crate gtk;
#[cfg(feature = "gui")]
extern crate gdk_pixbuf;
#[cfg(feature = "gui")]
extern crate cairo;
#[cfg(feature = "gui")]
extern crate gdk;

use std::env::args;
#[macro_use]
mod util;
mod decoder;
mod data;
mod cpu;
mod memory;
#[cfg(feature = "gui")]
mod gui;
mod io;
#[cfg(feature = "gui")]
mod widgets;
mod ports;
mod interrupts;
use std::ffi::OsString;
use cpu::{Cpu};
use memory::{Memory};

fn main() {
    if args().count() != 2 {
        println!("usage: vm <program>");
        return;
    }

    #[cfg(not(feature = "gui"))]
    {
        let mem = Memory::new(OsString::from(args().nth(1).expect("There must be an argument")), None);
        let mut cpu = Cpu::new(mem, true);
        while cpu.step() {}
    }

    #[cfg(feature = "gui")]
    {
        use widgets::{Button, Led, Poti, Seg7};
        use io::IO;

        let io = IO::new();
        let mut gui = gui::init();
        let _ = Led::new(&mut gui, "red0", 0xff, 0x00, 0x00, io.vcc.clone(), io.p[3][7].clone());
        let _ = Led::new(&mut gui, "green0", 0x00, 0xff, 0x00, io.vcc.clone(), io.p[2][0].clone());
        let _ = Led::new(&mut gui, "yellow0", 0xff, 0xff, 0x00, io.vcc.clone(), io.p[2][1].clone());
        let _ = Led::new(&mut gui, "blue0", 0x00, 0x00, 0xff, io.vcc.clone(), io.p[2][6].clone());
        let _ = Led::new(&mut gui, "red1", 0xff, 0x00, 0x00, io.vcc.clone(), io.p[2][7].clone());
        let _ = Led::new(&mut gui, "green1", 0x00, 0xff, 0x00, io.vcc.clone(), io.p[0][7].clone());
        let _ = Led::new(&mut gui, "yellow1", 0xff, 0xff, 0x00, io.vcc.clone(), io.p[0][6].clone());
        let _ = Led::new(&mut gui, "blue1", 0x00, 0x00, 0xff, io.vcc.clone(), io.p[0][5].clone());
        let _ = Button::new(&mut gui, "button0", io.p[3][3].clone());
        let _ = Button::new(&mut gui, "button1", io.p[3][2].clone());
        let _ = Poti::new(&mut gui, "potentiometer", io.gnd.clone(), io.p[0][1].clone(), io.vcc.clone());
        let _ = Poti::new(&mut gui, "light sensor", io.gnd.clone(), io.p[0][0].clone(), io.vcc.clone());
        let mut dis2 = Seg7::new(&mut gui, "dis2", io.p[1][1].clone(), io.p[1][0].clone(), io.p[3][1].clone(),
                         io.p[1][6].clone(), io.vcc.clone(), io.p[1][5].clone(), io.p[1][4].clone(),
                         io.p[3][1].clone(), io.p[1][3].clone(), io.p[1][2].clone());
        let mut dis1 = Seg7::new(&mut gui, "dis1", io.p[1][1].clone(), io.p[1][0].clone(), io.p[3][0].clone(),
                             io.p[1][6].clone(), io.vcc.clone(), io.p[1][5].clone(), io.p[1][4].clone(),
                             io.p[3][0].clone(), io.p[1][3].clone(), io.p[1][2].clone());

        io.nreset.set(io::LOW);
        io.gnd.set(io::LOW);
        io.vcc.set(io::HIGH);

        let mem = Memory::new(OsString::from(args().nth(1).expect("There must be an argument")), Some(&io));
        let mut cpu = Cpu::new(mem, false);

        while gui.step() {
            for _ in 0..100 {
                cpu.step();
            }
            dis1.step();
            dis2.step();
        }
    }
}
