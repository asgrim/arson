use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Orientation, Toolbar, ToolButton, ScrolledWindow, ShadowType, TextView, Menu, MenuBar, MenuItem, AboutDialog, Image, FileChooserDialog, FileChooserAction, ResponseType};
use serde_json::Value;

fn main() {
    let app = Application::builder()
        .application_id("com.jamestitcumb.ArsonGtk")
        .build();

    app.connect_activate(|app| {
        let fire_emoji_icon = Image::from_file("fire-emoji.ico");

        let win = ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(600)
            .title("Arson JSON")
            .icon(&fire_emoji_icon.pixbuf().clone().unwrap())
            .build();

        let v_box = Box::builder()
            .visible(true)
            .orientation(Orientation::Vertical)
            .build();
        win.add(&v_box);

        let file_menu = Menu::new();
        let file_quit_item = MenuItem::builder()
            .label("Quit")
            .build();
        let file_open_item = MenuItem::builder()
            .label("Open...")
            .build();
        file_menu.append(&file_open_item);
        file_menu.append(&file_quit_item);

        let help_menu = Menu::new();
        let help_about_item = MenuItem::builder()
            .label("About")
            .build();
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
            let text_view = text_view.clone();
            move |_| {
                let buffer = text_view.buffer().unwrap();
                let (start, end) = buffer.bounds();
                let pretty_json = buffer.text(&start, &end, true).unwrap();

                // @todo handle invalid JSON - currently we crash - https://github.com/asgrim/arson/issues/1
                let v: Value = serde_json::from_str(pretty_json.as_str()).unwrap();

                buffer.set_text(&serde_json::to_string_pretty(&v).unwrap());
            }
        });

        minify_button.connect_clicked({
            let text_view = text_view.clone();
            move |_| {
                let buffer = text_view.buffer().unwrap();
                let (start, end) = buffer.bounds();
                let ugly_json = buffer.text(&start, &end, true).unwrap();

                // @todo handle invalid JSON - currently we crash - https://github.com/asgrim/arson/issues/1
                let v: Value = serde_json::from_str(ugly_json.as_str()).unwrap();

                buffer.set_text(&serde_json::to_string(&v).unwrap());
            }
        });

        file_quit_item.connect_activate({
            let win = win.clone();
            move |_| {
                win.close();
            }
        });

        file_open_item.connect_activate({
            let win = win.clone();
            move |_| {
                let file_chooser = FileChooserDialog::builder()
                    .title("Open File")
                    .parent(&win)
                    .action(FileChooserAction::Open)
                    .build();

                file_chooser.add_buttons(&[
                    ("Open", ResponseType::Ok),
                    ("Cancel", ResponseType::Cancel),
                ]);

                file_chooser.connect_response({
                    let text_view = text_view.clone();
                    move |file_chooser, response| {
                        if response == ResponseType::Ok {
                            let filename = file_chooser.filename().expect("Couldn't get filename");
                            let file = File::open(filename).expect("Couldn't open file");

                            let mut reader = BufReader::new(file);
                            let mut contents = String::new();
                            let _ = reader.read_to_string(&mut contents);

                            text_view.buffer()
                                .unwrap()
                                .set_text(&contents);
                        }
                        file_chooser.close();
                    }
                });

                file_chooser.show_all();
            }
        });

        help_about_item.connect_activate({
            let win = win.clone();
            let fire_emoji_icon = fire_emoji_icon.clone();
            move |_| {
                let p = AboutDialog::new();
                p.set_website_label(Some("github.com/asgrim/arson"));
                p.set_website(Some("https://github.com/asgrim/arson"));
                p.set_authors(&["James Titcumb"]);
                p.set_title("About Arson");
                p.set_transient_for(Some(&win));
                p.set_logo(Some(&fire_emoji_icon.pixbuf().clone().unwrap()));
                p.show_all();
            }
        });

        win.show_all();
    });

    app.run();
}
