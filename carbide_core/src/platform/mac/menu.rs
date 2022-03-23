use cocoa::appkit::{NSEventModifierFlags, NSMenu, NSMenuItem};
use cocoa::base::{id, nil, NO, selector, YES};
use cocoa::foundation::{NSAutoreleasePool, NSString};
use objc::{msg_send, sel, sel_impl, class};
use objc::runtime::{Object, Sel};
use carbide_core::prelude::{MenuItem, MenuKind};


use crate::event::{HotKey, ModifierKey};
use crate::platform::mac::{from_nsstring, make_nsstring, process_name};
use crate::widget::Menu;

pub fn set_application_menu(menu: &Vec<Menu>) {
    let default_menu = convert_menu(&Menu::new("".to_string()));

    let app_menu = default_app_menu();
    add_sub_menu(default_menu, app_menu);

    for menu in menu {
        let menu = convert_menu(menu);
        add_sub_menu(default_menu, menu)
    }
    let app: id = unsafe { msg_send![class!(NSApplication), sharedApplication] };
    let () = unsafe { msg_send![app, setMainMenu: default_menu] };
    //let upper_menu_item = unsafe { NSMenuItem::alloc(nil).autorelease() };
    //add_sub_menu(upper_menu_item, menu);
    //add_item(current_menu, upper_menu_item);
}

pub fn convert_menu(menu: &Menu) -> id {
    let converted = make_menu(menu.name());

    match &menu.kind {
        Some(kind) => {
            match kind {
                MenuKind::Window => {
                    let app: id = unsafe { msg_send![class!(NSApplication), sharedApplication] };
                    let () = unsafe { msg_send![app, setWindowsMenu: converted] };
                }
                MenuKind::Help => {
                    let app: id = unsafe { msg_send![class!(NSApplication), sharedApplication] };
                    let () = unsafe { msg_send![app, setHelpMenu: converted] };
                }
            }
        }
        None => (),
    }
    for item in menu.items() {
        match item {
            MenuItem::Item { id, name, hotkey, enabled, selected } => {
                let menu_item = make_menu_item(&name, hotkey.clone(), "", *enabled);
                add_item(converted, menu_item);
            }
            MenuItem::Separator => {
                let sep = make_separator();
                add_item(converted, sep);
            },
            MenuItem::SubMenu { menu } => {
                let sub = convert_menu(menu);
                add_sub_menu(converted, sub);
            }
        }
    }

    converted
}

struct KeyEquivalent<'a> {
    key: &'a str,
    masks: Option<NSEventModifierFlags>,
}

fn default_app_menu() -> id {
    let process_name = from_nsstring(process_name());
    let mut about_name = "About ".to_string();
    about_name.push_str(&process_name);

    let menu = make_menu("");
    let about = make_internal_menu_item(
        make_nsstring(&about_name),
        selector("orderFrontStandardAboutPanel:"),
        None,
        true
    );

    let preferences = make_internal_menu_item(
        make_nsstring("Preferences..."),
        selector("undefined_selector:"),
        Some(KeyEquivalent {
            key: ",",
            masks: None, //Some(NSEventModifierFlags::NSCommandKeyMask)
        }),
        false
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
        true
    );

    let hide_others = make_internal_menu_item(
        make_nsstring("Hide Others"),
        selector("hideOtherApplications:"),
        Some(KeyEquivalent {
            key: "h",
            masks: Some(
                NSEventModifierFlags::NSAlternateKeyMask
                    | NSEventModifierFlags::NSCommandKeyMask,
            ),
        }),
        true
    );

    let show_all = make_internal_menu_item(
        make_nsstring("Show All"),
        selector("unhideAllApplications:"),
        None,
        true
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
        true
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
}

fn add_item(menu: id, item: id) {
    unsafe {
        let () = msg_send![menu, addItem: item];
    }
}

fn add_sub_menu(menu: id, submenu: id) {
    let item = make_title_menu_item(&menu_title(submenu));
    unsafe {
        let () = msg_send![item, setSubmenu: submenu];
    }
    add_item(menu, item)
}

fn menu_title(menu: id) -> String {
    unsafe {
        let s: id = msg_send![menu, title];
        from_nsstring(s)
    }
}

fn make_separator() -> id {
    unsafe {
        msg_send![class!(NSMenuItem), separatorItem]
    }
}

fn make_menu(title: &str) -> id {
    let title = make_nsstring(title);
    unsafe {
        let menu = NSMenu::alloc(nil).initWithTitle_(title).autorelease();
        let () = msg_send![menu, setAutoenablesItems: NO];
        menu
    }
}

fn make_title_menu_item(title: &str) -> id {
    make_menu_item(title, None, "", true)
}

fn make_empty_menu_item() -> id {
    make_menu_item("", None, "", true)
}

fn make_internal_menu_item(
    title: id,
    selector: Sel,
    key_equivalent: Option<KeyEquivalent<'_>>,
    enabled: bool
) -> id {
    unsafe {
        let (key, masks) = match key_equivalent {
            Some(ke) => (NSString::alloc(nil).init_str(ke.key), ke.masks),
            None => (NSString::alloc(nil).init_str(""), None),
        };
        let item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(title, selector, key);
        if let Some(masks) = masks {
            item.setKeyEquivalentModifierMask_(masks)
        }

        if enabled {
            let () = msg_send![item, setEnabled: YES];
        } else {
            let () = msg_send![item, setEnabled: NO];
        }

        item
    }
}

fn make_menu_item(title: &str, key: Option<HotKey>, code: &str, enabled: bool) -> id {
    unsafe {
        let key_combination = code;//key.map(HotKey::key_equivalent).unwrap_or("".to_string());

        let menu_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
            make_nsstring(title),
            sel!(handleMenuItem:),
            make_nsstring(key_combination),
        ).autorelease();

        if let Some(mask) = key.map(HotKey::key_modifier_mask) {
            let () = msg_send![menu_item, setKeyEquivalentModifierMask: mask];
        }

        if enabled {
            let () = msg_send![menu_item, setEnabled: YES];
        } else {
            let () = msg_send![menu_item, setEnabled: NO];
        }

        let () = msg_send![menu_item, setState: NO];
        menu_item
    }
}

impl HotKey {
    /*/// Return the string value of this hotkey, for use with Cocoa `NSResponder`
    /// objects.
    ///
    /// Returns the empty string if no key equivalent is known.
    fn key_equivalent(self) -> String {
        match &self.key {
            Key::Return => Key::Return.code().to_string(),//"\u{0003}",
            _ => {
                eprintln!("no key equivalent for {:?}", self);
                "".to_string()
            }
        }

        /*
        KbKey::Character(t) => t,

            // from NSText.h
            KbKey::Enter => "\u{0003}",
            KbKey::Backspace => "\u{0008}",
            KbKey::Delete => "\u{007f}",
            // from NSEvent.h
            KbKey::Insert => "\u{F727}",
            KbKey::Home => "\u{F729}",
            KbKey::End => "\u{F72B}",
            KbKey::PageUp => "\u{F72C}",
            KbKey::PageDown => "\u{F72D}",
            KbKey::PrintScreen => "\u{F72E}",
            KbKey::ScrollLock => "\u{F72F}",
            KbKey::ArrowUp => "\u{F700}",
            KbKey::ArrowDown => "\u{F701}",
            KbKey::ArrowLeft => "\u{F702}",
            KbKey::ArrowRight => "\u{F703}",
            KbKey::F1 => "\u{F704}",
            KbKey::F2 => "\u{F705}",
            KbKey::F3 => "\u{F706}",
            KbKey::F4 => "\u{F707}",
            KbKey::F5 => "\u{F708}",
            KbKey::F6 => "\u{F709}",
            KbKey::F7 => "\u{F70A}",
            KbKey::F8 => "\u{F70B}",
            KbKey::F9 => "\u{F70C}",
            KbKey::F10 => "\u{F70D}",
            KbKey::F11 => "\u{F70E}",
            KbKey::F12 => "\u{F70F}",
            //KbKey::F13            => "\u{F710}",
            //KbKey::F14            => "\u{F711}",
            //KbKey::F15            => "\u{F712}",
            //KbKey::F16            => "\u{F713}",
            //KbKey::F17            => "\u{F714}",
            //KbKey::F18            => "\u{F715}",
            //KbKey::F19            => "\u{F716}",
            //KbKey::F20            => "\u{F717}",
        */
    }*/

    fn key_modifier_mask(self) -> NSEventModifierFlags {
        let mods = self.modifier;
        let mut flags = NSEventModifierFlags::empty();
        if mods.contains(ModifierKey::SHIFT) {
            flags.insert(NSEventModifierFlags::NSShiftKeyMask);
        }
        if mods.contains(ModifierKey::GUI) {
            flags.insert(NSEventModifierFlags::NSCommandKeyMask);
        }
        if mods.contains(ModifierKey::ALT) {
            flags.insert(NSEventModifierFlags::NSAlternateKeyMask);
        }
        if mods.contains(ModifierKey::CTRL) {
            flags.insert(NSEventModifierFlags::NSControlKeyMask);
        }
        flags
    }
}