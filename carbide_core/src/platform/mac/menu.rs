use cocoa::appkit::{NSEventModifierFlags, NSMenuItem};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::NSAutoreleasePool;
use objc::{class, msg_send, sel, sel_impl};

use carbide_core::event::Key;

use crate::event::{HotKey, ModifierKey};
use crate::platform::mac::make_nsstring;

pub fn make_menu_item(title: &str, key: Option<HotKey>, code: String) -> id {
    unsafe {
        let key_combination = code;//key.map(HotKey::key_equivalent).unwrap_or("".to_string());

        let menu_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
            make_nsstring(title),
            sel!(handleMenuItem:),
            make_nsstring(&key_combination),
        ).autorelease();

        if let Some(mask) = key.map(HotKey::key_modifier_mask) {
            let () = msg_send![menu_item, setKeyEquivalentModifierMask: mask];
        }

        let () = msg_send![menu_item, setEnabled: YES];
        let () = msg_send![menu_item, setState: NO];
        menu_item
    }
}

impl HotKey {
    /// Return the string value of this hotkey, for use with Cocoa `NSResponder`
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
    }

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