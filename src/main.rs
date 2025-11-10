use gtk::gdk::ffi::gdk_screen_height;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::gio::{Cancellable, MemoryInputStream};
use gtk::glib::Bytes;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box, Orientation, Overlay, Paned, PolicyType, ScrolledWindow,
    ShadowType, TextView, ToolButton, Toolbar, WindowPosition,
};
use std::cell::Cell;
use std::rc::Rc;

mod file_mgt;
mod json_editor;
mod menu_bar;
mod tree_view;

fn remove_double_newline_action(text_view: TextView) {
    let buffer = text_view.buffer().unwrap();
    let (start, end) = buffer.bounds();
    let text_content = buffer.text(&start, &end, true).unwrap();

    buffer.set_text(text_content.as_str().replace("\n\n", "").as_str());
}

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

        let toolbar = Toolbar::builder().visible(true).expand(false).build();
        v_box.add(&toolbar);

        let pretty_button = ToolButton::builder()
            .visible(true)
            .label("Pretty")
            .tooltip_text("Format this JSON to be human-readable")
            .is_important(true)
            .use_underline(true)
            .icon_name("format-indent-more")
            .build();
        toolbar.add(&pretty_button);

        let minify_button = ToolButton::builder()
            .visible(true)
            .label("Minify")
            .tooltip_text("Remove extra spaces and newlines to compact space")
            .is_important(true)
            .use_underline(true)
            .icon_name("format-justify-fill")
            .build();
        toolbar.add(&minify_button);

        let remove_double_newlines = ToolButton::builder()
            .visible(true)
            .label("Remove \\n\\n")
            .tooltip_text("Remove double-newlines, i.e. \\n\\n")
            .is_important(true)
            .use_underline(true)
            .icon_name("error-correct-symbolic")
            .build();
        toolbar.add(&remove_double_newlines);

        let unescape_json_string = ToolButton::builder()
            .visible(true)
            .label("Unescape")
            .tooltip_text("Remove backslashes from escaped characters")
            .is_important(true)
            .use_underline(true)
            .icon_name("document-properties-symbolic")
            .build();
        toolbar.add(&unescape_json_string);

        let escape_json_string = ToolButton::builder()
            .visible(true)
            .label("Escape")
            .tooltip_text("Escape current text into a JSON string (adds quotes and backslashes)")
            .is_important(true)
            .use_underline(true)
            .icon_name("document-save-as-symbolic")
            .build();
        toolbar.add(&escape_json_string);

        // Toggle Tree View panel visibility
        let toggle_tree_button = ToolButton::builder()
            .visible(true)
            .label("Toggle Tree")
            .tooltip_text("Show/Hide the tree view panel")
            .is_important(true)
            .use_underline(true)
            .icon_name("view-list-symbolic")
            .build();
        toolbar.add(&toggle_tree_button);

        paned.set_visible(true);
        v_box.add(&paned);

        let scrolled_window = ScrolledWindow::builder()
            .visible(true)
            .can_focus(true)
            .shadow_type(ShadowType::In)
            .expand(true)
            .build();
        paned.pack1(&scrolled_window, true, true);

        let text_view = TextView::builder()
            .visible(true)
            .can_focus(true)
            .monospace(true)
            .build();
        scrolled_window.add(&text_view);

        pretty_button.connect_clicked({
            let win = win.clone();
            let text_view = text_view.clone();
            move |_| json_editor::prettify_json_action(win.clone(), text_view.clone())
        });

        minify_button.connect_clicked({
            let win = win.clone();
            let text_view = text_view.clone();
            move |_| json_editor::minify_json_action(win.clone(), text_view.clone())
        });

        remove_double_newlines.connect_clicked({
            let text_view = text_view.clone();
            move |_| remove_double_newline_action(text_view.clone())
        });

        unescape_json_string.connect_clicked({
            let win = win.clone();
            let text_view = text_view.clone();
            move |_| json_editor::unescape_json_action(win.clone(), text_view.clone())
        });

        escape_json_string.connect_clicked({
            let win = win.clone();
            let text_view = text_view.clone();
            move |_| json_editor::escape_json_action(win.clone(), text_view.clone())
        });

        menu_bar::attach_listeners(&menu_bar, &win.clone(), &text_view.clone(), &fire_emoji_icon_pb.clone());

        win.connect_scroll_event({
            let text_view = text_view.clone();
            move |_, event_key| {
                json_editor::ctrl_scroll_resize_text_view_action(
                    event_key.clone(),
                    text_view.clone(),
                )
            }
        });

        win.connect_key_press_event({
            let text_view = text_view.clone();
            move |_, event_key| {
                json_editor::ctrl_plus_minus_text_view_action(event_key.clone(), text_view.clone())
            }
        });

        win.connect_show({
            let text_view = text_view.clone();
            move |_| {
                text_view.grab_focus();
            }
        });

        let (tree_view, model) = tree_view::factory_tree_view();
        let invalid_overlay = tree_view::factory_invalid_overlay();

        // Create an overlay to show invalid JSON message over the tree
        let overlay = Overlay::builder().visible(true).build();
        // Wrap the tree view in a scrolled window so it doesn't grow the main window
        let scroller = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
        scroller.set_policy(PolicyType::Automatic, PolicyType::Automatic);
        scroller.set_hexpand(true);
        scroller.set_vexpand(true);
        scroller.add(&tree_view);

        overlay.add(&scroller);
        overlay.add_overlay(&invalid_overlay);

        paned.pack2(&overlay, true, true);

        // Track Tree visibility state
        let tree_visible = Rc::new(Cell::new(true));

        // Wire toggle button to show/hide the tree panel and update state
        toggle_tree_button.connect_clicked({
            let overlay = overlay.clone();
            let tree_visible = tree_visible.clone();
            let text_view = text_view.clone();
            let model = model.clone();
            let tree_view = tree_view.clone();
            let invalid_overlay = invalid_overlay.clone();
            move |_| {
                let currently_visible = tree_visible.get();
                if currently_visible {
                    overlay.hide();
                    tree_visible.set(false);
                } else {
                    overlay.show();
                    tree_visible.set(true);
                    // Rebuild once when showing
                    tree_view::build_tree_from_text(
                        text_view.clone(),
                        model.clone(),
                        tree_view.clone(),
                        invalid_overlay.clone(),
                    );
                }
            }
        });

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

        // Auto-update tree view when text changes (disabled when hidden)
        if let Some(buffer) = text_view.buffer() {
            buffer.connect_changed({
                let text_view = text_view.clone();
                let model = model.clone();
                let tree_view = tree_view.clone();
                let invalid_overlay = invalid_overlay.clone();
                let tree_visible = tree_visible.clone();
                move |_| {
                    if tree_visible.get() {
                        tree_view::build_tree_from_text(
                            text_view.clone(),
                            model.clone(),
                            tree_view.clone(),
                            invalid_overlay.clone(),
                        );
                    }
                }
            });
        }

        win.show_all();
        text_view.buffer().unwrap().set_text("{}");
    });

    app.run();
}
