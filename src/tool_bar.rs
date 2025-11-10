use crate::tree_view::TreeViewState;
use crate::{json_editor, tree_view};
use gtk::prelude::*;
use gtk::{ToolButton, Toolbar};

pub struct ToolBarState {
    pub toolbar: Toolbar,
    pretty_button: ToolButton,
    minify_button: ToolButton,
    remove_double_newlines: ToolButton,
    unescape_json_string: ToolButton,
    escape_json_string: ToolButton,
    toggle_tree_button: ToolButton,
}

pub fn factory_tool_bar() -> ToolBarState {
    let toolbar = Toolbar::builder().visible(true).expand(false).build();

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

    let toggle_tree_button = ToolButton::builder()
        .visible(true)
        .label("Toggle Tree")
        .tooltip_text("Show/Hide the tree view panel")
        .is_important(true)
        .use_underline(true)
        .icon_name("view-list-symbolic")
        .build();
    toolbar.add(&toggle_tree_button);

    ToolBarState {
        toolbar,
        pretty_button,
        minify_button,
        remove_double_newlines,
        unescape_json_string,
        escape_json_string,
        toggle_tree_button,
    }
}

pub fn attach_listeners(
    tool_bar: &ToolBarState,
    win: &gtk::ApplicationWindow,
    json_editor: json_editor::JsonEditorState,
    tree_view: TreeViewState,
) {
    tool_bar.pretty_button.connect_clicked({
        let win = win.clone();
        let json_editor = json_editor.clone();
        move |_| json_editor::prettify_json_action(win.clone(), json_editor.clone())
    });

    tool_bar.minify_button.connect_clicked({
        let win = win.clone();
        let json_editor = json_editor.clone();
        move |_| json_editor::minify_json_action(win.clone(), json_editor.clone())
    });

    tool_bar.remove_double_newlines.connect_clicked({
        let json_editor = json_editor.clone();
        move |_| json_editor::remove_double_newline_action(json_editor.clone())
    });

    tool_bar.unescape_json_string.connect_clicked({
        let win = win.clone();
        let json_editor = json_editor.clone();
        move |_| json_editor::unescape_json_action(win.clone(), json_editor.clone())
    });

    tool_bar.escape_json_string.connect_clicked({
        let win = win.clone();
        let json_editor = json_editor.clone();
        move |_| json_editor::escape_json_action(win.clone(), json_editor.clone())
    });

    tool_bar.toggle_tree_button.connect_clicked({
        let json_editor = json_editor.clone();
        let tree_view = tree_view.clone();
        move |_| tree_view::toggle_tree_view_visibility(json_editor.clone(), tree_view.clone())
    });
}
