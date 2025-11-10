use gtk::glib::Value;
use gtk::prelude::*;
use gtk::{
    Align, CellRendererText, CssProvider, Justification, Label, Orientation, Overlay, PolicyType,
    ScrolledWindow, TextView, TreePath, TreeStore, TreeView, TreeViewColumn,
    STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use serde_json::Value as JsonValue;
use std::cell::Cell;
use std::rc::Rc;

pub struct TreeViewState {
    pub overlay: Overlay,
    pub tree_view: TreeView,
    pub invalid_overlay: gtk::Box,
    visible: Rc<Cell<bool>>,
    model: TreeStore,
}

pub fn toggle_tree_view_visibility(text_view: &TextView, tree_view: &TreeViewState) {
    let currently_visible = tree_view.visible.get();
    if currently_visible {
        tree_view.overlay.hide();
        tree_view.visible.set(false);
    } else {
        tree_view.overlay.show();
        tree_view.visible.set(true);

        build_tree_from_text(text_view, tree_view);
    }
}

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

pub fn factory_tree_view() -> std::rc::Rc<TreeViewState> {
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

    let invalid_overlay = factory_invalid_overlay();

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

    Rc::new(TreeViewState {
        overlay,
        tree_view,
        invalid_overlay,
        visible: Rc::new(Cell::new(true)),
        model,
    })
}

fn factory_invalid_overlay() -> gtk::Box {
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

pub fn build_tree_from_text(text_view: &TextView, tree_view: &TreeViewState) {
    if (tree_view.visible.get()) == false {
        return;
    }

    let buffer = text_view.buffer().unwrap();
    let (start, end) = buffer.bounds();
    let text = buffer.text(&start, &end, true).unwrap();

    let parsed: Result<JsonValue, _> = serde_json::from_str(text.as_str());
    match parsed {
        Ok(v) => {
            // Valid JSON: hide overlay and populate tree
            tree_view.invalid_overlay.hide();
            tree_view.model.clear();
            append_json_value(&tree_view.model, None, "🔥", &v);

            // Expand the root node (first top-level row) by default
            let path = TreePath::new_first();
            tree_view.tree_view.expand_row(&path, false);
        }
        Err(e) => {
            // Invalid JSON: clear tree, show overlay with message
            if let Some(label) = tree_view
                .invalid_overlay
                .children()
                .into_iter()
                .find_map(|w| w.downcast::<Label>().ok())
            {
                label.set_label(&format!("Invalid JSON\n{}", e));
            }
            tree_view.invalid_overlay.show();
        }
    }
}
