use gtk;
use gtk::prelude::*;
use gtk::{Window, WindowType, Orientation, Widget, WidgetExt, Frame};

pub struct Gui {
    hbox: gtk::Box,
}

static mut RUNNING: bool = true;

pub fn init() -> Gui {
    gtk::init().expect("Failed to initialize GTK");

    let window = Window::new(WindowType::Toplevel);
    window.set_title("AVR-VM");
    window.set_border_width(10);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        unsafe { RUNNING = false; }
        Inhibit(false)
    });

    let hbox = gtk::Box::new(Orientation::Horizontal, 0);
    window.add(&hbox);
    window.show_all();

    Gui { hbox: hbox }
}

impl Gui {
    pub fn step(&self) -> bool {
        gtk::main_iteration_do(false);
        unsafe { RUNNING }
    }

    pub fn add<T: IsA<Widget> + WidgetExt + IsA<gtk::Object>>(&mut self, name: &str, w: &T) {
        let frame = Frame::new(Some(name));
        frame.add(w);
        w.show();
        frame.show();
        self.hbox.pack_start(&frame, false, false, 0);
    }
}
