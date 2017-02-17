#![allow(dead_code)]
use libc::{c_int, c_char};
use std::ptr::null;

static mut RUNNING: bool = true;

extern "C" {
    fn gui_init(argcp: *mut c_int, argvp: *const *const *const c_char, deleted: extern fn());
    fn gui_step();
}

extern "C" fn deleted_callback() {
    unsafe { RUNNING = false; }
}

pub fn init() {
    let mut c = 0;
    unsafe {
        gui_init(&mut c, null(), deleted_callback);
    }
}

pub fn step() -> bool {
    unsafe {
        gui_step();
        RUNNING
    }
}
