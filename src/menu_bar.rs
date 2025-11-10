use crate::file_mgt;
use gtk::prelude::*;
use gtk::{AboutDialog, Menu, MenuBar, MenuItem};

pub struct MenuBarState {
    pub menu_bar: MenuBar,
    file_open_item: MenuItem,
    file_open_url_item: MenuItem,
    file_quit_item: MenuItem,
    help_github_item: MenuItem,
    help_about_item: MenuItem,
}

pub fn factory_menu_bar() -> MenuBarState {
    let file_menu = Menu::new();
    let file_open_item = MenuItem::builder().label("Open...").build();
    let file_open_url_item = MenuItem::builder().label("Open URL...").build();
    let file_quit_item = MenuItem::builder().label("Quit").build();
    file_menu.append(&file_open_item);
    file_menu.append(&file_open_url_item);
    file_menu.append(&file_quit_item);

    let help_menu = Menu::new();
    let help_github_item = MenuItem::builder().label("GitHub Issues").build();
    let help_about_item = MenuItem::builder().label("About").build();
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

    MenuBarState {
        menu_bar,
        file_open_item,
        file_open_url_item,
        file_quit_item,
        help_github_item,
        help_about_item,
    }
}

pub fn attach_listeners(
    menu_bar: &MenuBarState,
    win: &gtk::ApplicationWindow,
    text_view: &gtk::TextView,
    fire_emoji_icon_pb: &gtk::gdk_pixbuf::Pixbuf,
) {
    menu_bar.file_quit_item.connect_activate({
        let win = win.clone();
        move |_| {
            win.close();
        }
    });

    menu_bar.file_open_item.connect_activate({
        let win = win.clone();
        let text_view = text_view.clone();
        move |_| file_mgt::file_open_item_action(win.clone(), text_view.clone())
    });

    menu_bar.file_open_url_item.connect_activate({
        let win = win.clone();
        let text_view = text_view.clone();
        move |_| file_mgt::file_open_url_item_action(win.clone(), text_view.clone())
    });

    menu_bar.help_about_item.connect_activate({
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

    menu_bar.help_github_item.connect_activate(|_| {
        let _ = open::that("https://github.com/asgrim/arson/issues");
    });
}
