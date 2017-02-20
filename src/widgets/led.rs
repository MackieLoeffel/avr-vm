use gtk;
use gtk::prelude::*;
use gdk_pixbuf::Pixbuf;
use gui::Gui;
use io::Wire;
use std::rc::Rc;
use std::cell::RefCell;

// 0 is Colorspace RBG, see http://gtk-rs.org/docs/gdk_pixbuf_sys/constant.GDK_COLORSPACE_RGB.html
const GDK_COLORSPACE_RGB: i32 = 0;

struct LedState {
    image: gtk::Image,
    icon: [Pixbuf; 2],

    wire_a: Rc<Wire>,
    wire_b: Rc<Wire>,
}

pub struct Led { }

impl Led {
    pub fn new(gui: &mut Gui, name: &str, r: u8, g: u8, b: u8, porta: Rc<Wire>, portb: Rc<Wire>) -> Led {
        assert!((r as u32) + (g as u32) + (b as u32) >= 0xff);

        let mut icon = unsafe {[
            Pixbuf::new(GDK_COLORSPACE_RGB, true, 8, 16, 16).unwrap(),
            Pixbuf::new(GDK_COLORSPACE_RGB, true, 8, 16, 16).unwrap(),
        ]};
        draw_led(&mut icon[0], 0, 0, 0);
        draw_led(&mut icon[1], r, g, b);

        let image = gtk::Image::new();
        image.set_from_pixbuf(Some(&icon[1]));
        image.show();

        gui.add(name, &image);

        let state = Rc::new(RefCell::new(LedState {
            image: image,
            icon: icon,
            wire_a: porta.clone(),
            wire_b: portb.clone(),
        }));

        state.borrow_mut().update();
        let statea = state.clone();
        porta.add_listener(move || {statea.borrow_mut().update();});
        let stateb = state.clone();
        portb.add_listener(move || {stateb.borrow_mut().update();});

        Led { }
    }
}

impl LedState {
    fn update(&mut self) {
        self.image.set_from_pixbuf(Some(&self.icon[if self.wire_b.mv() < self.wire_a.mv() { 1 } else { 0 }]));
    }
}

fn draw_led(icon: &mut Pixbuf, r: u8, g: u8, b: u8) {
    assert_eq!(icon.get_n_channels(), 4);
    assert_eq!(icon.get_colorspace(), GDK_COLORSPACE_RGB);
    assert_eq!(icon.get_bits_per_sample(), 8);
    assert_eq!(icon.get_has_alpha(), true);

    let width = icon.get_width();
    let height = icon.get_height();

    for x in 0..width {
        for y in 0..height {
            let d2 = (x-width/2)*(x-width/2) + (y-height/2)*(y-height/2);
            if d2 <= ((width-4)/2)*((width-4)/2) {
                // Circle
                icon.put_pixel(x, y, r, g, b, 0xff);
            } else if d2 < (width / 2) * (width / 2) {
                // Frame
                icon.put_pixel(x, y, 0, 0, 0, 0xff);
            } else {
                // Outside
                icon.put_pixel(x, y, 0, 0, 0, 0x00);
            }
        }
    }
}
