use gtk::{ApplicationWindow, ButtonsType, FileChooserAction, FileChooserDialog, MessageDialog, MessageType, ResponseType, TextView, WindowPosition};
use std::fs::File;
use std::io::{BufReader, Read};
use serde_json::Value;
use gtk::prelude::{ContainerExt, DialogExt, DialogExtManual, EntryExt, FileChooserExt, GtkWindowExt, TextBufferExt, TextViewExt, WidgetExt};

pub fn file_open_item_action(win: ApplicationWindow, text_view: TextView)
{
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

pub fn file_open_url_item_action(win: ApplicationWindow, text_view: TextView)
{
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