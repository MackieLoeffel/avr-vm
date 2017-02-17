use io::Wire;

pub enum WidgetType<'a> {
    Led(u8, u8, u8, &'a Wire, &'a Wire),
    Button(&'a Wire),
    Poti(&'a Wire, &'a Wire, &'a Wire),
    Seg7(&'a Wire, &'a Wire, &'a Wire, &'a Wire, &'a Wire,
    &'a Wire, &'a Wire, &'a Wire, &'a Wire, &'a Wire)
}

pub struct Widget<'a> {
    typ: WidgetType<'a>,
    // cdata: *mut c_void
}

impl<'a> Widget<'a> {
    pub fn new(name: &str, typ: WidgetType<'a>) -> Widget<'a> {
        // let cname = CString::new(name).expect("expected correct string");
        // let cdata = match typ {
            // WidgetType::Led(r, g, b, port_a, port_c) => unsafe {
                // led_create(cname.as_ptr(), r as c_int, g as c_int, b as c_int, port_a, port_c)
            // },
            // WidgetType::Button(port) => unsafe {
                // button_create(cname.as_ptr(), port)
            // },
            // WidgetType::Poti(port_left, port_middle, port_right) => unsafe {
                // poti_create(cname.as_ptr(), port_left, port_middle, port_right)
            // },
            // WidgetType::Seg7(port_e, port_d, port_anode0, port_c, port_dp, port_b, port_a, port_anode1, port_f, port_g) => unsafe {
                // seg7_create(cname.as_ptr(), port_e, port_d, port_anode0, port_c, port_dp, port_b, port_a, port_anode1, port_f, port_g)
            // }
        // };
        Widget {typ: typ,
                // cdata: cdata
        }
    }

    pub fn step(&self) {
        match self.typ {
            // WidgetType::Seg7(..) => unsafe { seg7_step(self.cdata); },
            _ => {}
        }
    }
}

// impl<'a> Drop for Widget<'a> {
    // fn drop(&mut self) {
        // match self.typ {
            // WidgetType::Led(..) => unsafe { led_destroy(self.cdata); },
            // WidgetType::Button(..) => unsafe { button_destroy(self.cdata); },
            // WidgetType::Poti(..) => unsafe { poti_destroy(self.cdata); },
            // WidgetType::Seg7(..) => unsafe { seg7_destroy(self.cdata); },
        // }
    // }
// }
