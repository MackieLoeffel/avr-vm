use std::mem;
use std::ptr;
use std::ops::Deref;

pub const LOGIC_0: u32 = ((0 & 0xffff) << 0) | (0xffff << 16);
pub const LOGIC_1: u32 = ((5000 & 0xffff) << 0) | ( 0xffff << 16);
pub const LOGIC_Z: u32 = 0xffff;
pub const LOGIC_H: u32 = ((5000 & 0xffff) << 0) | ( 1 << 16);

static ext_data: WireData = WireData { mv: 0 };

struct WireData {
    mv: u16
}

pub struct Wire {
    // logic: *mut sig_std_logic,
    // we need a constant address, so we cannot allocate this on the stack
    // because it may be moved there
    // data: Box<WireData>,
    // same as above
    // funcs: Box<sig_std_logic_funcs>
}

impl Wire {
    pub fn new() -> Wire {
        // unsafe {
            // let funcs = Box::new(sig_std_logic_funcs {
                // boolean_or_set: None,
                // consume: None,
                // set_ext: Some(wire_set_cb),
                // set_extN: None,
                // std_logic_set: None,
                // std_logic_setN: None,
                // supply: None,
                // supply_ext: None,
            // });
            // let mut w = Wire {
                // logic: sig_std_logic_create(CString::new("wire").unwrap().as_ptr()),
                // data: Box::new(WireData {mv: 0}),
                // funcs: funcs
            // };
            // sig_std_logic_connect_out(w.logic, &ext_data, LOGIC_Z);
            // sig_std_logic_connect_out(w.logic, &mut w.data as &mut WireData, LOGIC_Z);
            // sig_std_logic_connect_in(w.logic, &mut w.data as &mut WireData, &mut w.funcs as &mut sig_std_logic_funcs);
        // w
        Wire {}
    }

    #[inline(always)]
    pub fn set(&self, value: u32) {
        unimplemented!();
        // unsafe {
            // sig_std_logic_set(self.logic, &self.data as &WireData, value);
        // }
    }

    // this function is needed, if the value should be read back from the cpu itself
    // because the own value is ignored, when calculating the value of the wire
    #[inline(always)]
    pub fn set_ext(&self, value: u32) {
        unimplemented!();
        // unsafe {
            // sig_std_logic_set(self.logic, &ext_data, value);
        // }
    }

    #[inline(always)]
    pub fn set_bool(&self, value: bool) {
        unimplemented!();
        // self.set(if value {LOGIC_1} else {LOGIC_0});
    }

    #[inline(always)]
    pub fn mv(&self) -> u16 {
        unimplemented!();
        // self.data.mv
    }

    #[inline(always)]
    pub fn as_bin(&self) -> u8 {
        unimplemented!();
        // (self.mv() > 2500) as u8
    }
}

// unsafe extern "C" fn wire_set_cb(data: *mut WireData, value: u32) {
    // (*data).mv = value as u16;
    // println!("New mV: {}", (*data).mv);
// }

// impl Deref for Wire {
    // type Target = sig_std_logic;

    // fn deref(&self) -> &sig_std_logic {
        // unsafe { &*self.logic as &sig_std_logic}
    // }
// }

// impl Drop for Wire {
    // fn drop(&mut self) {
        // unsafe {
            // sig_std_logic_destroy(self.logic);
        // }
    // }
// }

pub struct IO {
    pub nreset: Wire,
    pub vcc: Wire,
    pub gnd: Wire,
    pub p: [[Wire; 8]; 4]
}

impl IO {
    pub fn new() -> IO {
        // see http://stackoverflow.com/a/31361031
        let mut p: [[Wire; 8]; 4] = unsafe { [mem::uninitialized(),
                                              mem::uninitialized(),
                                              mem::uninitialized(),
                                              mem::uninitialized()] };
        for outer in p.iter_mut() {
            for elem in outer.iter_mut() {
                unsafe {
                    ptr::write(elem, Wire::new());
                }
            }
        }

        IO {
            nreset: Wire::new(),
            vcc: Wire::new(),
            gnd: Wire::new(),
            p: p
        }
    }
}
