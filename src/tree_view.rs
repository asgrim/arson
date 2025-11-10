use gtk::glib::Value;
use gtk::prelude::*;
use gtk::{
    Align, CellRendererText, CssProvider, Justification, Label, Orientation, TextView, TreePath,
    TreeStore, TreeView, TreeViewColumn, STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use serde_json::Value as JsonValue;

fn append_json_value(model: &TreeStore, parent: Option<&gtk::TreeIter>, key: &str, v: &JsonValue) {
    match v {
        JsonValue::Object(map) => {
            let iter = model.append(parent);
            model.set_value(&iter, 0, &Value::from(key));
            model.set_value(&iter, 1, &Value::from("{object}"));
            for (k, val) in map.iter() {
                append_json_value(model, Some(&iter), k.as_str(), val);
            }
        }
        JsonValue::Array(arr) => {
            let iter = model.append(parent);
            model.set_value(&iter, 0, &Value::from(key));
            model.set_value(&iter, 1, &Value::from("[list]"));
            for (i, val) in arr.iter().enumerate() {
                let idx_key = format!("[{}]", i);
                append_json_value(model, Some(&iter), &idx_key, val);
            }
        }
        JsonValue::String(s) => {
            let iter = model.append(parent);
            model.set_value(&iter, 0, &Value::from(key));
            model.set_value(&iter, 1, &Value::from(s.as_str()));
        }
        JsonValue::Number(n) => {
            let iter = model.append(parent);
            model.set_value(&iter, 0, &Value::from(key));
            model.set_value(&iter, 1, &Value::from(n.to_string()));
        }
        JsonValue::Bool(b) => {
            let iter = model.append(parent);
            model.set_value(&iter, 0, &Value::from(key));
            model.set_value(&iter, 1, &Value::from(b.to_string()));
        }
        JsonValue::Null => {
            let iter = model.append(parent);
            model.set_value(&iter, 0, &Value::from(key));
            model.set_value(&iter, 1, &Value::from("null"));
        }
    }
}

pub fn factory_tree_view() -> (TreeView, TreeStore) {
    let tree_view = TreeView::builder().visible(true).expand(true).build();

    // Two columns: Key and Value
    let column = TreeViewColumn::new();
    column.set_title("Key");
    let cell = CellRendererText::new();
    gtk::prelude::CellLayoutExt::pack_start(&column, &cell, true);
    gtk::prelude::TreeViewColumnExt::add_attribute(&column, &cell, "text", 0);
    tree_view.append_column(&column);

    let column = TreeViewColumn::new();
    column.set_title("Value");
    let cell = CellRendererText::new();
    gtk::prelude::CellLayoutExt::pack_start(&column, &cell, true);
    gtk::prelude::TreeViewColumnExt::add_attribute(&column, &cell, "text", 1);
    tree_view.append_column(&column);

    // TreeStore with two string columns
    let model = TreeStore::new(&[String::static_type(), String::static_type()]);
    tree_view.set_model(Some(&model));
    tree_view.set_headers_visible(true);

    (tree_view, model)
}

pub fn factory_invalid_overlay() -> gtk::Box {
    // Translucent grey overlay message for invalid JSON
    let invalid_overlay = gtk::Box::new(Orientation::Vertical, 8);
    invalid_overlay.set_visible(false);
    invalid_overlay.set_halign(Align::Fill);
    invalid_overlay.set_valign(Align::Fill);
    invalid_overlay.set_hexpand(true);
    invalid_overlay.set_vexpand(true);

    let css = CssProvider::new();
    let _ = css.load_from_data(
        br#"
.invalid-overlay {
    background: rgba(0, 0, 0, 0.30);
}

.invalid-overlay label {
    color: white;
    font-weight: bold;
    font-size: 16pt;
}
"#,
    );
    invalid_overlay
        .style_context()
        .add_provider(&css, STYLE_PROVIDER_PRIORITY_APPLICATION);
    invalid_overlay.style_context().add_class("invalid-overlay");

    let invalid_label = Label::new(Some("Invalid JSON"));
    // Center horizontally and vertically
    invalid_label.set_halign(Align::Center);
    invalid_label.set_valign(Align::Center);
    // Allow the label to take extra space so centering works inside the Box
    invalid_label.set_hexpand(true);
    invalid_label.set_vexpand(true);
    // Center multi-line text within the label's own allocation
    invalid_label.set_xalign(0.5);
    invalid_label.set_justify(Justification::Center);
    invalid_overlay.add(&invalid_label);
    invalid_overlay.hide();

    invalid_overlay
}

pub fn build_tree_from_text(
    text_view: TextView,
    model: TreeStore,
    tree_view: TreeView,
    invalid_overlay: gtk::Box,
) {
    let buffer = text_view.buffer().unwrap();
    let (start, end) = buffer.bounds();
    let text = buffer.text(&start, &end, true).unwrap();

    let parsed: Result<JsonValue, _> = serde_json::from_str(text.as_str());
    match parsed {
        Ok(v) => {
            // Valid JSON: hide overlay and populate tree
            invalid_overlay.hide();
            model.clear();
            append_json_value(&model, None, "🔥", &v);

            // Expand the root node (first top-level row) by default
            let path = TreePath::new_first();
            tree_view.expand_row(&path, false);
        }
        Err(e) => {
            // Invalid JSON: clear tree, show overlay with message
            if let Some(label) = invalid_overlay
                .children()
                .into_iter()
                .find_map(|w| w.downcast::<Label>().ok())
            {
                label.set_label(&format!("Invalid JSON\n{}", e));
            }
            invalid_overlay.show();
        }
    }
}
