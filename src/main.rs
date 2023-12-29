use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Orientation, Toolbar, ToolButton, ScrolledWindow, ShadowType, TextView, Menu, MenuBar, MenuItem, AboutDialog, FileChooserDialog, FileChooserAction, ResponseType, MessageDialog, MessageType, ButtonsType, WindowPosition};
use gtk::gdk::ffi::gdk_screen_height;
use gtk::gdk_pixbuf::{Pixbuf};
use gtk::gio::{Cancellable, MemoryInputStream};
use gtk::glib::Bytes;
use serde_json::Value;

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
            move |_| {
                let buffer = text_view.buffer().unwrap();
                let (start, end) = buffer.bounds();
                let pretty_json = buffer.text(&start, &end, true).unwrap();

                let v: Value = match serde_json::from_str(pretty_json.as_str()) {
                    Ok(v) => v,
                    Err(e) => {
                        let error_dialog = MessageDialog::builder()
                            .transient_for(&win)
                            .window_position(WindowPosition::CenterOnParent)
                            .message_type(MessageType::Warning)
                            .buttons(ButtonsType::Ok)
                            .title("JSON was invalid")
                            .text(format!("The current text was not valid JSON.\n\n{}", e))
                            .build();
                        error_dialog.connect_response(move |error_dialog, _| {
                            error_dialog.close();
                        });
                        error_dialog.run();
                        return
                    },
                };

                buffer.set_text(&serde_json::to_string_pretty(&v).unwrap());
            }
        });

        minify_button.connect_clicked({
            let win = win.clone();
            let text_view = text_view.clone();
            move |_| {
                let buffer = text_view.buffer().unwrap();
                let (start, end) = buffer.bounds();
                let ugly_json = buffer.text(&start, &end, true).unwrap();

                let v: Value = match serde_json::from_str(ugly_json.as_str()) {
                    Ok(v) => v,
                    Err(e) => {
                        let error_dialog = MessageDialog::builder()
                            .transient_for(&win)
                            .window_position(WindowPosition::CenterOnParent)
                            .message_type(MessageType::Warning)
                            .buttons(ButtonsType::Ok)
                            .title("JSON was invalid")
                            .text(format!("The current text was not valid JSON.\n\n{}", e))
                            .build();
                        error_dialog.connect_response(move |error_dialog, _| {
                            error_dialog.close();
                        });
                        error_dialog.run();
                        return
                    },
                };

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
            let text_view = text_view.clone();
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

        file_open_url_item.connect_activate({
            let win = win.clone();
            let text_view = text_view.clone();
            move |_| {
                let url_entry_dialog = gtk::Dialog::builder()
                    .transient_for(&win)
                    .window_position(WindowPosition::CenterOnParent)
                    .title("Open JSON From URL")
                    .build();
                let url_entry_label = gtk::Label::builder()
                    .label("Enter the URL containing JSON to be opened")
                    .build();
                let url_entry_text = gtk::Entry::builder()
                    .build();
                url_entry_dialog.content_area().add(&url_entry_label);
                url_entry_dialog.content_area().add(&url_entry_text);
                url_entry_dialog.add_button("Open", ResponseType::Ok);

                url_entry_dialog.connect_response({
                    let win = win.clone();
                    let text_view = text_view.clone();
                    move |url_entry_dialog, response| {
                        if response == ResponseType::Ok {
                            let url_text = url_entry_text.text();

                            let body = match reqwest::blocking::get(url_text.as_str()) {
                                Ok(http_response) => http_response.text().unwrap(),
                                Err(e) => {
                                    let error_dialog = MessageDialog::builder()
                                        .transient_for(&win)
                                        .window_position(WindowPosition::CenterOnParent)
                                        .message_type(MessageType::Warning)
                                        .buttons(ButtonsType::Ok)
                                        .title("JSON was invalid")
                                        .text(format!("The URL {} could not be loaded.\n\n{}", url_text.as_str(), e))
                                        .build();
                                    error_dialog.connect_response(move |error_dialog, _| {
                                        error_dialog.close();
                                    });
                                    error_dialog.run();
                                    return
                                },
                            };

                            let _: Value = match serde_json::from_str(body.as_str()) {
                                Ok(v) => v,
                                Err(e) => {
                                    let error_dialog = MessageDialog::builder()
                                        .transient_for(&win)
                                        .window_position(WindowPosition::CenterOnParent)
                                        .message_type(MessageType::Warning)
                                        .buttons(ButtonsType::Ok)
                                        .title("JSON was invalid")
                                        .text(format!("The content from the URL {} was not valid JSON.\n\n{}", url_text.as_str(), e))
                                        .build();
                                    error_dialog.connect_response(move |error_dialog, _| {
                                        error_dialog.close();
                                    });
                                    error_dialog.run();
                                    return
                                },
                            };

                            let buffer = text_view.buffer().unwrap();
                            buffer.set_text(body.as_str());
                        }
                        url_entry_dialog.close();
                    }
                });

                url_entry_dialog.show_all();
            }
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

        win.show_all();
    });

    app.run();
}
