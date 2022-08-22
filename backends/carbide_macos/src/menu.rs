use cocoa::base::{id, nil};
use crate::string::NSString;
use cocoa::appkit::NSMenu as InnerNSMenu;
use cocoa::foundation::{NSAutoreleasePool, NSInteger};
use objc::{msg_send, class, sel, sel_impl};
use crate::id::Id;
use crate::menu_item::NSMenuItem;
use cocoa::base::NO;
use cocoa::base::YES;
use carbide_core::widget::{Menu, MenuItem};
use carbide_core::environment::Environment;
use crate::NSArray;

pub struct NSMenu {
    pub(crate) id: id,
}

impl NSMenu {
    pub fn new(title: &str) -> NSMenu {
        let ns_string = NSString::from(title);
        let id = unsafe {
            let menu = InnerNSMenu::alloc(nil).initWithTitle_(ns_string.id()).  autorelease();
            let () = msg_send![menu, setAutoenablesItems: NO];
            menu
        };
        NSMenu {
            id
        }
    }

    pub fn new_services_menu(title: &str) -> NSMenu {
        let services = NSMenu::new(title);

        let app: id = unsafe { msg_send![class!(NSApplication), sharedApplication] };
        let () = unsafe { msg_send![app, setServicesMenu: services.id()] };

        services
    }

    pub fn new_help_menu(title: &str) -> NSMenu {
        let help = NSMenu::new(title);

        let app: id = unsafe { msg_send![class!(NSApplication), sharedApplication] };
        let () = unsafe { msg_send![app, setHelpMenu: help.id()] };

        help
    }

    pub fn new_windows_menu(title: &str) -> NSMenu {
        let windows = NSMenu::new(title);

        let app: id = unsafe { msg_send![class!(NSApplication), sharedApplication] };
        let () = unsafe { msg_send![app, setWindowsMenu: windows.id()] };

        windows
    }

    pub fn set_as_main_menu(self) {
        let app: id = unsafe { msg_send![class!(NSApplication), sharedApplication] };

        let main_menu: id = unsafe { msg_send![app, mainMenu] };

        // Do manual cleanup of the previous menu. This is only cleaning up the
        // streams in the environment,
        if main_menu != nil {
            NSMenu {id: main_menu}.cleanup();
        }

        let () = unsafe { msg_send![app, setMainMenu: self.id()] };
    }

    pub fn cleanup(&self) {
        for menu_item in self.menu_items() {
            menu_item.cleanup()
        }
    }

    pub fn menu_items(&self) -> Vec<NSMenuItem> {
        unsafe {
            let items = NSArray {inner: msg_send![self.id(), itemArray]};

            let mut res = vec![];

            for index in 0..items.len() {
                let item = NSMenuItem {
                    id: items.at(index),
                    responder: nil,
                };
                res.push(item)
            }

            res
        }
    }

    pub fn default_menu() -> NSMenu {
        todo!()
        /*
        let process_name = from_nsstring(process_name());
    let mut about_name = "About ".to_string();
    about_name.push_str(&process_name);

    let menu = make_menu("");
    let about = make_internal_menu_item(
        make_nsstring(&about_name),
        selector("orderFrontStandardAboutPanel:"),
        None,
        true,
    );

    let preferences = make_internal_menu_item(
        make_nsstring("Preferences..."),
        selector("undefined_selector:"),
        Some(KeyEquivalent {
            key: ",",
            masks: None, //Some(NSEventModifierFlags::NSCommandKeyMask)
        }),
        false,
    );

    let services = make_menu("Services");

    let app: id = unsafe { msg_send![class!(NSApplication), sharedApplication] };
    let () = unsafe { msg_send![app, setServicesMenu: services] };

    let mut hide_item_name = "Hide ".to_string();
    hide_item_name.push_str(&process_name);
    let hide_item = make_internal_menu_item(
        make_nsstring(&hide_item_name),
        selector("hide:"),
        Some(KeyEquivalent {
            key: "h",
            masks: None,
        }),
        true,
    );

    let hide_others = make_internal_menu_item(
        make_nsstring("Hide Others"),
        selector("hideOtherApplications:"),
        Some(KeyEquivalent {
            key: "h",
            masks: Some(
                NSEventModifierFlags::NSAlternateKeyMask | NSEventModifierFlags::NSCommandKeyMask,
            ),
        }),
        true,
    );

    let show_all = make_internal_menu_item(
        make_nsstring("Show All"),
        selector("unhideAllApplications:"),
        None,
        true,
    );

    let mut quit_item_name = "Quit ".to_string();
    quit_item_name.push_str(&process_name);
    let quit_item = make_internal_menu_item(
        make_nsstring(&quit_item_name),
        selector("terminate:"),
        Some(KeyEquivalent {
            key: "q",
            masks: None,
        }),
        true,
    );

    add_item(menu, about);
    add_item(menu, make_separator());
    add_item(menu, preferences);
    add_item(menu, make_separator());
    add_sub_menu(menu, services);
    add_item(menu, make_separator());
    add_item(menu, hide_item);
    add_item(menu, hide_others);
    add_item(menu, show_all);
    add_item(menu, make_separator());
    add_item(menu, quit_item);

    menu
        */
    }

    pub fn add_item(mut self, item: NSMenuItem) -> Self {
        unsafe {
            let () = msg_send![self.id(), addItem: item.id()];
        }
        self
    }

    pub fn insert_item_at_index(mut self, item: NSMenuItem, index: u32) -> Self {
        let index = NSInteger::from(index);
        unsafe {
            let () = msg_send![self.id(), insertItem: item.id() atIndex: index];
        }
        self
    }

    pub fn remove_item(mut self, item: NSMenuItem) -> Self {
        item.cleanup();
        unsafe {
            let () = msg_send![self.id(), removeItem: item.id()];
        }
        self
    }

    pub fn remove_item_at_index(mut self, index: u32) -> Self {
        let index = NSInteger::from(index);
        unsafe {
            let () = msg_send![self.id(), removeItemAtIndex: index];
        }
        self
    }

    pub fn remove_all_items(mut self) -> Self {
        self.cleanup();
        unsafe {
            let () = msg_send![self.id(), removeAllItems];
        }
        self
    }

    pub fn from(menu: &Menu, env: &mut Environment) -> Self {
        let mut new_menu = NSMenu::new(menu.name());

        for item in menu.items() {
            match item {
                MenuItem::Item { id, name, hotkey, enabled, selected, action } => {
                    let item = NSMenuItem::new(name, *hotkey)
                        .set_enabled(*enabled)
                        .set_action(action.clone(), env);

                    new_menu = new_menu.add_item(item);
                }
                MenuItem::Separator => {
                    new_menu = new_menu.add_item(NSMenuItem::separator());
                }
                MenuItem::SubMenu { menu } => {
                    let item = NSMenuItem::new(menu.name(), None)
                        .set_submenu(NSMenu::from(menu, env));

                    new_menu = new_menu.add_item(item);
                }
            }
        }

        new_menu
    }
}

impl Id for NSMenu {
    fn id(&self) -> id {
        self.id
    }
}