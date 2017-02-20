use gtk;
use gtk::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use io::Wire;
use gui::Gui;

struct PotiState {
    adjustment: gtk::Adjustment,

    wire_left: Rc<Wire>,
    wire_middle: Rc<Wire>,
    wire_right: Rc<Wire>,
}

pub struct Poti { }

impl Poti {
    pub fn new(gui: &mut Gui, name: &str, wire_left: Rc<Wire>, wire_middle: Rc<Wire>, wire_right: Rc<Wire>) -> Poti {
        let adjustment = gtk::Adjustment::new(0.0, 0.0, 110.0, 5.0, 10.0, 10.0);
        let scale = gtk::Scale::new(gtk::Orientation::Horizontal, Some(&adjustment));
        scale.show();
        gui.add(name, &scale);

        let state = Rc::new(RefCell::new(PotiState {
            adjustment: adjustment,
            wire_left: wire_left,
            wire_middle: wire_middle,
            wire_right: wire_right,
        }));
        state.borrow_mut().update();

        let state_adj = state.clone();
        state.borrow().adjustment.connect_value_changed(move |_| { state_adj.borrow_mut().update(); });
        let state_left = state.clone();
        state.borrow().wire_left.add_listener(move || { state_left.borrow_mut().update(); });
        let state_right = state.clone();
        state.borrow().wire_right.add_listener(move || { state_right.borrow_mut().update(); });

        Poti { }
    }
}

impl PotiState {
    fn update(&mut self) {
        self.wire_middle.set((self.wire_left.mv() as f64
                              + (self.wire_right.mv() as f64 - self.wire_left.mv() as f64)
                              * self.adjustment.get_value() / 100.0) as u16);
    }
}
