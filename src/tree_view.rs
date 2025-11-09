use gtk::prelude::*;
use gtk::{ApplicationWindow, ButtonsType, CellRendererText, MessageDialog, MessageType, TextView, TreePath, TreeStore, TreeView, TreeViewColumn, WindowPosition};
use gtk::glib::Value;
use serde_json::Value as JsonValue;

fn append_json_value(model: &TreeStore, parent: Option<&gtk::TreeIter>, key: &str, v: &JsonValue) {
    match v {
        JsonValue::Object(map) => {
            let iter = model.append(parent);
            model.set_value(&iter, 0, &Value::from(key));
            model.set_value(&iter, 1, &Value::from("{}"));
            for (k, val) in map.iter() {
                append_json_value(model, Some(&iter), k.as_str(), val);
            }
        }
        JsonValue::Array(arr) => {
            let iter = model.append(parent);
            model.set_value(&iter, 0, &Value::from(key));
            model.set_value(&iter, 1, &Value::from("[]"));
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
    let tree_view = TreeView::builder()
        .visible(true)
        .expand(true)
        .build();

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

pub fn build_tree_from_text(win: ApplicationWindow, text_view: TextView, model: TreeStore, tree_view: TreeView) {
    let buffer = text_view.buffer().unwrap();
    let (start, end) = buffer.bounds();
    let text = buffer.text(&start, &end, true).unwrap();

    let parsed: Result<JsonValue, _> = serde_json::from_str(text.as_str());
    let v = match parsed {
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

    model.clear();
    append_json_value(&model, None, "root", &v);

    // Expand the root node (first top-level row) by default
    let path = TreePath::new_first();
    tree_view.expand_row(&path, false);
}
