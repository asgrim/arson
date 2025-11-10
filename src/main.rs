use gtk::gdk::ffi::gdk_screen_height;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::gio::{Cancellable, MemoryInputStream};
use gtk::glib::Bytes;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box, Orientation, Paned, ScrolledWindow, ShadowType, TextView,
    WindowPosition,
};
use std::cell::Cell;
use std::rc::Rc;

mod file_mgt;
mod json_editor;
mod menu_bar;
mod tool_bar;
mod tree_view;

fn main() {
    let app = Application::builder()
        .application_id("com.jamestitcumb.ArsonGtk")
        .build();

    app.connect_activate(|app| {
        let stream =
            MemoryInputStream::from_bytes(&Bytes::from(include_bytes!("../fire-emoji.ico")));
        let fire_emoji_icon_pb = Pixbuf::from_stream(&stream, Cancellable::NONE).unwrap();

        let screen_height = unsafe { gdk_screen_height() } as f64;
        let win_height = (screen_height * 0.7).round();

        let win = ApplicationWindow::builder()
            .application(app)
            .default_width((win_height * 1.33).round() as i32)
            .default_height(win_height as i32)
            .window_position(WindowPosition::Center)
            .title("Arson JSON")
            .icon(&fire_emoji_icon_pb.clone())
            .build();

        let v_box = Box::builder()
            .visible(true)
            .orientation(Orientation::Vertical)
            .build();
        win.add(&v_box);

        let paned = Paned::new(Orientation::Horizontal);

        let menu_bar = menu_bar::factory_menu_bar();
        v_box.pack_start(&menu_bar.menu_bar, false, false, 0);

        let tool_bar = tool_bar::factory_tool_bar();
        v_box.add(&tool_bar.toolbar);

        paned.set_visible(true);
        v_box.add(&paned);

        let json_editor = json_editor::factory_json_editor();
        paned.pack1(&json_editor.scrolled_window, true, true);

        let tree_view = tree_view::factory_tree_view();

        tool_bar::attach_listeners(
            &tool_bar,
            &win.clone(),
            json_editor.clone(),
            tree_view.clone(),
        );
        menu_bar::attach_listeners(
            &menu_bar,
            &win.clone(),
            json_editor.clone(),
            &fire_emoji_icon_pb.clone(),
        );

        win.connect_scroll_event({
            let json_editor = json_editor.clone();
            move |_, event_key| {
                json_editor::ctrl_scroll_resize_text_view_action(
                    event_key.clone(),
                    json_editor.clone(),
                )
            }
        });

        win.connect_key_press_event({
            let json_editor = json_editor.clone();
            move |_, event_key| {
                json_editor::ctrl_plus_minus_text_view_action(
                    event_key.clone(),
                    json_editor.clone(),
                )
            }
        });

        win.connect_show({
            let json_editor = json_editor.clone();
            move |_| {
                json_editor::focus(json_editor.clone());
            }
        });

        paned.pack2(&tree_view.overlay, true, true);

        let init_done = Rc::new(Cell::new(false));
        paned.connect_size_allocate({
            let paned = paned.clone();
            let init_done = init_done.clone();
            move |_, alloc| {
                if !init_done.get() {
                    paned.set_position(((alloc.width() as f64) * 0.7).round() as i32);
                    init_done.set(true);
                }
            }
        });

        win.show_all();
        json_editor::init_text_buffer(json_editor.clone());
    });

    app.run();
}
