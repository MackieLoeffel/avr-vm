static mut RUNNING: bool = true;

// extern "C" fn deleted_callback() {
    // unsafe { RUNNING = false; }
// }

pub fn init() {
    // let mut c = 0;
    // unsafe {
        // gui_init(&mut c, null(), deleted_callback);
    // }
}

pub fn step() -> bool {
    // unsafe {
        // gui_step();
        // RUNNING
    // }
    true
}
