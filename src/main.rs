use gtk::prelude::*;
use gtk::{AboutDialog, Application, ApplicationWindow, Box, Menu, MenuBar, MenuItem, Orientation, ScrolledWindow, ShadowType, TextView, ToolButton, Toolbar, WindowPosition};
use gtk::gdk::ffi::gdk_screen_height;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::gio::{Cancellable, MemoryInputStream};
use gtk::glib::Bytes;
mod file_mgt;
mod json_editor;

fn remove_double_newline_action(text_view: TextView)
{
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
        let stream = MemoryInputStream::from_bytes(
            &Bytes::from(include_bytes!("../fire-emoji.ico"))
        );
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

        let file_menu = Menu::new();
        let file_open_item = MenuItem::builder()
            .label("Open...")
            .build();
        let file_open_url_item = MenuItem::builder()
            .label("Open URL...")
            .build();
        let file_quit_item = MenuItem::builder()
            .label("Quit")
            .build();
        file_menu.append(&file_open_item);
        file_menu.append(&file_open_url_item);
        file_menu.append(&file_quit_item);

        let help_menu = Menu::new();
        let help_github_item = MenuItem::builder()
            .label("GitHub Issues")
            .build();
        let help_about_item = MenuItem::builder()
            .label("About")
            .build();
        help_menu.append(&help_github_item);
        help_menu.append(&help_about_item);

        let menu_bar = MenuBar::new();
        let file_item = MenuItem::builder()
            .label("File")
            .submenu(&file_menu)
            .build();
        let help_item = MenuItem::builder()
            .label("Help")
            .submenu(&help_menu)
            .build();
        menu_bar.append(&file_item);
        menu_bar.append(&help_item);
        v_box.pack_start(&menu_bar, false, false, 0);

        let toolbar = Toolbar::builder()
            .visible(true)
            .expand(false)
            .build();
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
            .icon_name("emblem-symbolic-link")
            .build();
        toolbar.add(&remove_double_newlines);

        let scrolled_window = ScrolledWindow::builder()
            .visible(true)
            .can_focus(true)
            .shadow_type(ShadowType::In)
            .expand(true)
            .build();
        v_box.add(&scrolled_window);

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

        file_quit_item.connect_activate({
            let win = win.clone();
            move |_| {
                win.close();
            }
        });

        file_open_item.connect_activate({
            let win = win.clone();
            let text_view = text_view.clone();
            move |_| file_mgt::file_open_item_action(win.clone(), text_view.clone())
        });

        file_open_url_item.connect_activate({
            let win = win.clone();
            let text_view = text_view.clone();
            move |_| file_mgt::file_open_url_item_action(win.clone(), text_view.clone())
        });

        help_about_item.connect_activate({
            let win = win.clone();
            let fire_emoji_icon_pb = fire_emoji_icon_pb.clone();
            move |_| {
                let p = AboutDialog::new();
                p.set_website_label(Some("github.com/asgrim/arson"));
                p.set_website(Some("https://github.com/asgrim/arson"));
                p.set_authors(&["James Titcumb"]);
                p.set_title("About Arson");
                p.set_transient_for(Some(&win));
                p.set_logo(Some(&fire_emoji_icon_pb.clone()));
                p.show_all();
            }
        });

        help_github_item.connect_activate(|_| {
            let _ = open::that("https://github.com/asgrim/arson/issues");
        });

        win.connect_scroll_event({
            let text_view = text_view.clone();
            move |_, event_key| json_editor::ctrl_scroll_resize_text_view_action(event_key.clone(), text_view.clone())
        });

        win.connect_key_press_event({
            let text_view = text_view.clone();
            move |_, event_key| json_editor::ctrl_plus_minus_text_view_action(event_key.clone(), text_view.clone())
        });

        win.connect_show({
            let text_view = text_view.clone();
            move |_| {
                text_view.grab_focus();
            }
        });

        win.show_all();
    });

    app.run();
}
