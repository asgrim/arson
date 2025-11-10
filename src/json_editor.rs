use crate::tree_view::TreeViewState;
use crate::{json_editor, tree_view};
use gtk::gdk::{EventKey, EventScroll, ModifierType, ScrollDirection};
use gtk::glib::Propagation;
use gtk::prelude::*;
use gtk::prelude::{
    CssProviderExt, DialogExt, GtkWindowExt, StyleContextExt, StyleContextExtManual, TextBufferExt,
    TextViewExt, WidgetExt,
};
use gtk::{
    ApplicationWindow, ButtonsType, CssProvider, MessageDialog, MessageType, ScrolledWindow,
    ShadowType, StateFlags, TextBuffer, TextView, WindowPosition,
    STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use serde_json::{json, Value};

#[derive(Clone)]
pub struct JsonEditorState {
    pub scrolled_window: ScrolledWindow,
    text_view: TextView,
}

pub fn factory_json_editor() -> JsonEditorState {
    let scrolled_window = ScrolledWindow::builder()
        .visible(true)
        .can_focus(true)
        .shadow_type(ShadowType::In)
        .expand(true)
        .build();

    let text_view = TextView::builder()
        .visible(true)
        .can_focus(true)
        .monospace(true)
        .build();
    scrolled_window.add(&text_view);

    JsonEditorState {
        scrolled_window,
        text_view,
    }
}

pub fn attach_listeners(json_editor: JsonEditorState, tree_view: TreeViewState) {
    if let Some(buffer) = json_editor.text_view.buffer() {
        let json_editor = json_editor.clone();
        let tree_view = tree_view.clone();
        buffer.connect_changed({
            move |_| {
                tree_view::build_tree_from_text(json_editor.clone(), tree_view.clone());
            }
        });
    }
}

pub fn init_text_buffer(json_editor: JsonEditorState) {
    retrieve_buffer(json_editor.clone()).set_text("{}");
}

pub fn focus(json_editor: JsonEditorState) {
    json_editor.text_view.grab_focus();
}

pub fn retrieve_buffer(json_editor: JsonEditorState) -> TextBuffer {
    json_editor.text_view.buffer().unwrap()
}

pub fn remove_double_newline_action(json_editor: JsonEditorState) {
    let buffer = json_editor.text_view.buffer().unwrap();
    let (start, end) = buffer.bounds();
    let text_content = buffer.text(&start, &end, true).unwrap();

    buffer.set_text(text_content.as_str().replace("\n\n", "").as_str());
}

pub fn ctrl_scroll_resize_text_view_action(
    event_key: EventScroll,
    json_editor: JsonEditorState,
) -> Propagation {
    if event_key.state().contains(ModifierType::CONTROL_MASK)
        && (event_key.direction() == ScrollDirection::Down
            || event_key.direction() == ScrollDirection::Up)
    {
        let mut dir = 1;

        if event_key.direction() == ScrollDirection::Down {
            dir = -1;
        }

        let cur_size = json_editor
            .text_view
            .style_context()
            .font(StateFlags::NORMAL)
            .size()
            / gtk::pango::SCALE;
        let css_override = CssProvider::new();
        let _ = css_override
            .load_from_data(format!("* {{ font-size: {}pt; }}", cur_size + dir).as_bytes());

        json_editor
            .text_view
            .style_context()
            .add_provider(&css_override, STYLE_PROVIDER_PRIORITY_APPLICATION);
    }

    Propagation::Proceed
}

pub fn ctrl_plus_minus_text_view_action(
    event_key: EventKey,
    json_editor: JsonEditorState,
) -> Propagation {
    if event_key.state().contains(ModifierType::CONTROL_MASK)
        && (event_key.hardware_keycode() == 86 || event_key.hardware_keycode() == 82)
    {
        let mut dir = -1;
        if event_key.hardware_keycode() == 86 {
            dir = 1;
        }

        let cur_size = json_editor
            .text_view
            .style_context()
            .font(StateFlags::NORMAL)
            .size()
            / gtk::pango::SCALE;
        let css_override = CssProvider::new();
        let _ = css_override
            .load_from_data(format!("* {{ font-size: {}pt; }}", cur_size + dir).as_bytes());

        json_editor
            .text_view
            .style_context()
            .add_provider(&css_override, STYLE_PROVIDER_PRIORITY_APPLICATION);
    }

    Propagation::Proceed
}

pub fn prettify_json_action(win: ApplicationWindow, json_editor: JsonEditorState) {
    let buffer = json_editor.text_view.buffer().unwrap();
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
            return;
        }
    };

    buffer.set_text(&serde_json::to_string_pretty(&v).unwrap());
}

pub fn minify_json_action(win: ApplicationWindow, json_editor: JsonEditorState) {
    let buffer = json_editor.text_view.buffer().unwrap();
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
            return;
        }
    };

    buffer.set_text(&serde_json::to_string(&v).unwrap());
}

pub fn unescape_json_action(win: ApplicationWindow, json_editor: JsonEditorState) {
    // Unescape a buffer that contains JSON encoded as a JSON string
    // Example input: {\"a\":1} or "{\"a\":1}"
    // Output: {"a":1}
    let buffer = json_editor.text_view.buffer().unwrap();
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
        }
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

pub fn escape_json_action(_win: ApplicationWindow, json_editor: JsonEditorState) {
    // Escape the current buffer into a JSON string
    // Example input: {"a":1}
    // Output: "{\"a\":1}"
    let buffer = json_editor.text_view.buffer().unwrap();
    let (start, end) = buffer.bounds();
    let current_text = buffer.text(&start, &end, true).unwrap();

    let escaped = serde_json::to_string(current_text.as_str()).unwrap();
    buffer.set_text(&escaped);
}
