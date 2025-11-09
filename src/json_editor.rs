use gtk::{ApplicationWindow, ButtonsType, CssProvider, MessageDialog, MessageType, StateFlags, TextView, WindowPosition, STYLE_PROVIDER_PRIORITY_APPLICATION};
use serde_json::Value;
use gtk::gdk::{EventKey, EventScroll, ModifierType, ScrollDirection};
use gtk::glib::Propagation;
use gtk::prelude::{CssProviderExt, DialogExt, GtkWindowExt, StyleContextExt, StyleContextExtManual, TextBufferExt, TextViewExt, WidgetExt};

pub fn ctrl_scroll_resize_text_view_action(event_key: EventScroll, text_view: TextView) -> Propagation
{
    if event_key.state().contains(ModifierType::CONTROL_MASK)
        && (event_key.direction() == ScrollDirection::Down || event_key.direction() == ScrollDirection::Up) {
        let mut dir = 1;

        if event_key.direction() == ScrollDirection::Down {
            dir = -1;
        }

        let cur_size = text_view.style_context().font(StateFlags::NORMAL).size() / gtk::pango::SCALE;
        let css_override = CssProvider::new();
        let _ = css_override.load_from_data(format!("* {{ font-size: {}pt; }}", cur_size + dir).as_bytes());

        text_view.style_context().add_provider(&css_override, STYLE_PROVIDER_PRIORITY_APPLICATION);
    }

    Propagation::Proceed
}

pub fn ctrl_plus_minus_text_view_action(event_key: EventKey, text_view: TextView) -> Propagation
{
    if event_key.state().contains(ModifierType::CONTROL_MASK) &&
        (event_key.hardware_keycode() == 86 || event_key.hardware_keycode() == 82)
    {
        let mut dir = -1;
        if event_key.hardware_keycode() == 86 {
            dir = 1;
        }

        let cur_size = text_view.style_context().font(StateFlags::NORMAL).size() / gtk::pango::SCALE;
        let css_override = CssProvider::new();
        let _ = css_override.load_from_data(format!("* {{ font-size: {}pt; }}", cur_size + dir).as_bytes());

        text_view.style_context().add_provider(&css_override, STYLE_PROVIDER_PRIORITY_APPLICATION);
    }

    Propagation::Proceed
}

pub fn prettify_json_action(win: ApplicationWindow, text_view: TextView)
{
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

pub fn minify_json_action(win: ApplicationWindow, text_view: TextView)
{
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

pub fn unescape_json_action(win: ApplicationWindow, text_view: TextView)
{
    // Unescape a buffer that contains JSON encoded as a JSON string
    // Example input: {\"a\":1} or "{\"a\":1}"
    // Output: {"a":1}
    let buffer = text_view.buffer().unwrap();
    let (start, end) = buffer.bounds();
    let current_text = buffer.text(&start, &end, true).unwrap();

    // 1) First, try to parse the whole buffer as a JSON string
    if let Ok(unescaped) = serde_json::from_str::<String>(current_text.as_str()) {
        buffer.set_text(&unescaped);
        return;
    }

    // 2) If that failed, try treating the buffer as the content of a JSON string
    //    by wrapping it in quotes (useful when the buffer is missing surrounding quotes)
    let wrapped = format!("\"{}\"", current_text);
    match serde_json::from_str::<String>(&wrapped) {
        Ok(unescaped) => {
            buffer.set_text(&unescaped);
        },
        Err(e) => {
            let error_dialog = MessageDialog::builder()
                .transient_for(&win)
                .window_position(WindowPosition::CenterOnParent)
                .message_type(MessageType::Warning)
                .buttons(ButtonsType::Ok)
                .title("Could not unescape text")
                .text(format!(
                    "The current text could not be interpreted as an escaped JSON string.\n\n{}",
                    e
                ))
                .build();
            error_dialog.connect_response(move |error_dialog, _| {
                error_dialog.close();
            });
            error_dialog.run();
        }
    }
}
