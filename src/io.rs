use std::mem;
use std::ptr;
use std::cell::RefCell;
use std::rc::Rc;

pub const HIGH: u16 = 5000;
pub const LOW: u16 = 0;

pub struct Wire {
    mv: RefCell<u16>,
    listeners: RefCell<Vec<Box<FnMut()>>>,
}

impl Wire {
    #[allow(dead_code)]
    pub fn new() -> Wire {
        Wire {
            mv: RefCell::new(LOW),
            listeners: RefCell::new(Vec::new()),
        }
    }

    #[allow(dead_code)]
    pub fn add_listener<F>(&self, f: F) where F: FnMut() + 'static {
        self.listeners.borrow_mut().push(Box::new(f));
    }

    #[inline(always)]
    pub fn set(&self, mv: u16) {
        *self.mv.borrow_mut() = mv;
        for listener in self.listeners.borrow_mut().iter_mut() {
            listener();
        }
    }

    #[inline(always)]
    pub fn mv(&self) -> u16 {
        *self.mv.borrow()
    }

    #[inline(always)]
    pub fn as_bin(&self) -> u8 {
        (self.mv() > LOW + (HIGH - LOW) / 2) as u8
    }
}

pub struct IO {
    pub nreset: Rc<Wire>,
    pub vcc: Rc<Wire>,
    pub gnd: Rc<Wire>,
    pub p: [[Rc<Wire>; 8]; 4]
}

impl IO {
    #[allow(dead_code)]
    pub fn new() -> IO {
        // see http://stackoverflow.com/a/31361031
        let mut p: [[Rc<Wire>; 8]; 4] = unsafe { [mem::uninitialized(),
                                              mem::uninitialized(),
                                              mem::uninitialized(),
                                              mem::uninitialized()] };
        for outer in p.iter_mut() {
            for elem in outer.iter_mut() {
                unsafe {
                    ptr::write(elem, Rc::new(Wire::new()));
                }
            }
        }

        IO {
            nreset: Rc::new(Wire::new()),
            vcc: Rc::new(Wire::new()),
            gnd: Rc::new(Wire::new()),
            p: p
        }
    }
}
