use gtk;
use gtk::prelude::*;
use gdk;
use cairo::Context;
use std::cell::RefCell;
use std::rc::Rc;
use io::Wire;
use gui::Gui;

const OFFX: f64 = 5.0;
const OFFY: f64 = 5.0;
const LX: f64 = 20.0;
const LY: f64 = 20.0;
const LSPACE: f64 = 16.0;

struct Seg7Data {
    drawing_area: gtk::DrawingArea,

    wires: [Rc<Wire>; 8],
    sum_seg: [u32; 8],
    state: [bool; 8],

    wire_anode0: Rc<Wire>,
    _wire_anode1: Rc<Wire>,
}

pub struct Seg7 {
    data: Rc<RefCell<Seg7Data>>,
    count: u32,
}

static SEG7_TABLE: [(fn(&Context, bool, f64, f64), f64, f64); 8] = [
    (draw_hor, 0. * LX, 0. * LY),
    (draw_vert, 1. * LX, 0. * LY),
    (draw_vert, 1. * LX, 1. * LY),
    (draw_hor, 0. * LX, 2. * LY),
    (draw_vert, 0. * LX, 1. * LY),
    (draw_vert, 0. * LX, 0. * LY),
    (draw_hor, 0. * LX, 1. * LY),
    (draw_dot, 1. * LX + LSPACE / 2., 2. * LY),
];

fn set_color(cr: &Context, state: bool) {
    if state {
        cr.set_source_rgb(0., 0., 0.);
    } else {
        cr.set_source_rgb(200. / 255., 200. / 255., 200. / 255.);
    }
}

fn draw_hor(cr: &Context, state: bool, x: f64, y: f64) {
    set_color(cr, state);
    cr.set_line_width(2.);
    cr.move_to(x + 4., y - 2.); cr.line_to(x + LX - 4., y - 2.);
    cr.move_to(x + 3., y - 1.); cr.line_to(x + LX - 3., y - 1.);
    cr.move_to(x + 2., y - 0.); cr.line_to(x + LX - 2., y - 0.);
    cr.move_to(x + 3., y + 1.); cr.line_to(x + LX - 3., y + 1.);
    cr.move_to(x + 4., y + 2.); cr.line_to(x + LX - 4., y + 2.);
    cr.stroke();
}

fn draw_vert(cr: &Context, state: bool, x: f64, y: f64) {
    set_color(cr, state);
    cr.set_line_width(2.);
    cr.move_to(x - 2., y + 4.); cr.line_to(x - 2., y + LY - 4.);
    cr.move_to(x - 1., y + 3.); cr.line_to(x - 1., y + LY - 3.);
    cr.move_to(x - 0., y + 2.); cr.line_to(x - 0., y + LY - 2.);
    cr.move_to(x + 1., y + 3.); cr.line_to(x + 1., y + LY - 3.);
    cr.move_to(x + 2., y + 4.); cr.line_to(x + 2., y + LY - 4.);
    cr.stroke();
}

fn draw_dot(cr: &Context, state: bool, x: f64, y: f64) {
    set_color(cr, state);
    cr.set_line_width(2.);
    cr.move_to(x - 2., y - 2.); cr.line_to(x + 2., y - 2.);
    cr.move_to(x - 2., y - 1.); cr.line_to(x + 2., y - 1.);
    cr.move_to(x - 2., y - 0.); cr.line_to(x + 2., y - 0.);
    cr.move_to(x - 2., y + 1.); cr.line_to(x + 2., y + 1.);
    cr.move_to(x - 2., y + 2.); cr.line_to(x + 2., y + 2.);
    cr.stroke();
}

impl Seg7 {
    pub fn new(gui: &mut Gui, name: &str,
               wire_e: Rc<Wire>, wire_d: Rc<Wire>, wire_anode0: Rc<Wire>, wire_c: Rc<Wire>,
               wire_dp: Rc<Wire>, wire_b: Rc<Wire>, wire_a: Rc<Wire>, wire_anode1: Rc<Wire>,
               wire_f: Rc<Wire>, wire_g: Rc<Wire>) -> Seg7 {
        let area = gtk::DrawingArea::new();
        area.set_size_request((OFFX + LX + OFFX) as i32, (OFFY + 2. * LY + OFFY) as i32);
        area.show();
        gui.add(name, &area);

        let data = Rc::new(RefCell::new(Seg7Data {
            drawing_area: area,
            wires: [wire_a, wire_b, wire_c, wire_d, wire_e, wire_f, wire_g, wire_dp],
            sum_seg: [0; 8],
            state: [false; 8],
            wire_anode0: wire_anode0,
            _wire_anode1: wire_anode1,
        }));

        let draw_data = data.clone();
        data.borrow().drawing_area.connect_draw(move |_, cr| {
            for i in 0..8 {
                SEG7_TABLE[i].0(&cr, draw_data.borrow().state[i], OFFX + SEG7_TABLE[i].1, OFFY + SEG7_TABLE[i].2);
            }
            Inhibit(false)
        });

        Seg7 {data: data, count: 0}
    }

    pub fn step(&mut self) {
        self.count += 1;
        let mut data = self.data.borrow_mut();
        for i in 0..8 {
            data.sum_seg[i] += ((2000 < data.wires[i].mv()) < (2000 < data.wire_anode0.mv())) as u32;
        }

        if self.count < 100 {
            return;
        }
        self.count = 0;

        for i in 0..8 {
            data.state[i] = 100 / 4 < data.sum_seg[i];
            data.sum_seg[i] = 0;
        }

        if let Some(window) = data.drawing_area.get_window() {
            window.invalidate_rect(&gdk::Rectangle {
                x: 0, y: 0, width: data.drawing_area.get_allocated_width(),
                height: data.drawing_area.get_allocated_height(),
            }, true);
        }
    }
}
