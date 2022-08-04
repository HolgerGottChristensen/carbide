use cocoa::appkit::NSEventModifierFlags;
use cocoa::base::{id, nil, selector};
use carbide_core::event::HotKey;
use crate::id::Id;
use cocoa::appkit::NSMenuItem as InnerNSMenuItem;
use cocoa::foundation::NSAutoreleasePool;
use objc::runtime::{NO, YES};
use objc::{msg_send, class, sel, sel_impl};
use crate::menu::NSMenu;
use crate::string::NSString;

pub struct NSMenuItem {
    id: id,
}

impl NSMenuItem {
    pub fn new(title: &str, action: &str, hot_key: Option<HotKey>) -> NSMenuItem {
        /*let (key, masks) = match key_equivalent {
            Some(ke) => (NSString::alloc(nil).init_str(ke.key), ke.masks),
            None => (NSString::alloc(nil).init_str(""), None),
        };*/

        let title = NSString::from(title);
        let key = NSString::from("");

        let id = unsafe {
            InnerNSMenuItem::alloc(nil)
                .initWithTitle_action_keyEquivalent_(title.id(), sel!(handleMenuItem:), key.id())
                .autorelease()
        };

        //if let Some(mask) = key.map(HotKey::key_modifier_mask) {
        //    let () = msg_send![menu_item, setKeyEquivalentModifierMask: mask];
        //}

        NSMenuItem {
            id
        }
    }

    pub fn separator() -> NSMenuItem {
        let id: id = unsafe { msg_send![class!(NSMenuItem), separatorItem] };

        NSMenuItem {
            id
        }
    }

    pub fn set_enabled(mut self, enabled: bool) -> NSMenuItem {
        let enabled = if enabled { YES } else { NO };
        unsafe {
            let () = msg_send![self.id(), setEnabled: enabled];
        }
        self
    }

    pub fn set_hidden(mut self, hidden: bool) -> NSMenuItem {
        let hidden = if hidden { YES } else { NO };
        unsafe {
            let () = msg_send![self.id(), setHidden: hidden];
        }
        self
    }

    pub fn set_submenu(mut self, menu: NSMenu) -> NSMenuItem {
        unsafe {
            let () = msg_send![self.id(), setSubmenu: menu];
        }
        self
    }

    pub fn set_action(mut self, action: &str) -> NSMenuItem {
        debug_assert!(action.ends_with(":"));

        unsafe {
            let sel = selector(action);
            let () = msg_send![self.id(), setAction: sel];
        }
        self
    }
}

impl Id for NSMenuItem {
    fn id(&self) -> id {
        self.id
    }
}

// struct KeyEquivalent<'a> {
//     key: &'a str,
//     masks: Option<NSEventModifierFlags>,
// }
//
// impl From<HotKey> for KeyEquivalent {
//     fn from(hot: HotKey) -> Self {
//         todo!()
//     }
// }