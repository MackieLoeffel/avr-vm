use libc::{c_char, c_int, c_void};
use io::sig_std_logic;
use std::ffi::CString;

extern "C" {
    fn led_create(name: *const c_char,
                      r: c_int, g: c_int,
                      b: c_int, port_a: *const sig_std_logic,
                      port_c: *const sig_std_logic) -> *mut c_void;
    fn led_destroy(_cpssp: *mut c_void);
    fn button_create(name: *const c_char, port: *const sig_std_logic) -> *mut c_void;
    fn button_destroy(_cpssp: *mut c_void);
    fn poti_create(name: *const c_char,
                   port_left: *const sig_std_logic,
                   port_middle: *const sig_std_logic,
                   port_right: *const sig_std_logic) -> *mut c_void;
    fn poti_destroy(_cpssp: *mut c_void);
    fn seg7_create(name: *const c_char,
                   port_e: *const sig_std_logic,
                   port_d: *const sig_std_logic,
                   port_anode0: *const sig_std_logic,
                   port_c: *const sig_std_logic,
                   port_dp: *const sig_std_logic,
                   port_b: *const sig_std_logic,
                   port_a: *const sig_std_logic,
                   port_anode1: *const sig_std_logic,
                   port_f: *const sig_std_logic,
                   port_g: *const sig_std_logic) -> *mut c_void;
    fn seg7_destroy(_cpssp: *mut c_void);
    fn seg7_step(_cpssp: *mut c_void);
}

pub enum WidgetType<'a> {
    Led(u8, u8, u8, &'a sig_std_logic, &'a sig_std_logic),
    Button(&'a sig_std_logic),
    Poti(&'a sig_std_logic, &'a sig_std_logic, &'a sig_std_logic),
    Seg7(&'a sig_std_logic, &'a sig_std_logic, &'a sig_std_logic, &'a sig_std_logic, &'a sig_std_logic,
    &'a sig_std_logic, &'a sig_std_logic, &'a sig_std_logic, &'a sig_std_logic, &'a sig_std_logic)
}

pub struct Widget<'a> {
    typ: WidgetType<'a>,
    cdata: *mut c_void
}

impl<'a> Widget<'a> {
    pub fn new(name: &str, typ: WidgetType<'a>) -> Widget<'a> {
        let cname = CString::new(name).expect("expected correct string");
        let cdata = match typ {
            WidgetType::Led(r, g, b, port_a, port_c) => unsafe {
                led_create(cname.as_ptr(), r as c_int, g as c_int, b as c_int, port_a, port_c)
            },
            WidgetType::Button(port) => unsafe {
                button_create(cname.as_ptr(), port)
            },
            WidgetType::Poti(port_left, port_middle, port_right) => unsafe {
                poti_create(cname.as_ptr(), port_left, port_middle, port_right)
            },
            WidgetType::Seg7(port_e, port_d, port_anode0, port_c, port_dp, port_b, port_a, port_anode1, port_f, port_g) => unsafe {
                seg7_create(cname.as_ptr(), port_e, port_d, port_anode0, port_c, port_dp, port_b, port_a, port_anode1, port_f, port_g)
            }
        };
        Widget {typ: typ, cdata: cdata}
    }

    pub fn step(&self) {
        match self.typ {
            WidgetType::Seg7(..) => unsafe { seg7_step(self.cdata); },
            _ => {}
        }
    }
}

impl<'a> Drop for Widget<'a> {
    fn drop(&mut self) {
        match self.typ {
            WidgetType::Led(..) => unsafe { led_destroy(self.cdata); },
            WidgetType::Button(..) => unsafe { button_destroy(self.cdata); },
            WidgetType::Poti(..) => unsafe { poti_destroy(self.cdata); },
            WidgetType::Seg7(..) => unsafe { seg7_destroy(self.cdata); },
        }
    }
}
