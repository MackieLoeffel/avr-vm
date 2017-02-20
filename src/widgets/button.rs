use gtk;
use gtk::prelude::*;
use gui::Gui;
use io::{Wire, HIGH, LOW};
use std::rc::Rc;

pub struct Button { }

impl Button {
    pub fn new(gui: &mut Gui, name: &str, wire: Rc<Wire>) -> Button {
        let button = gtk::Button::new();
        button.show();
        gui.add(name, &button);

        wire.set(HIGH);

        let wire_press = wire.clone();
        button.connect_button_press_event(move |_, _| {
            wire_press.set(LOW);
            Inhibit(false)
        });
        let wire_release = wire.clone();
        button.connect_button_release_event(move |_, _| {
            wire_release.set(HIGH);
            Inhibit(false)
        });

        Button { }
    }
}
