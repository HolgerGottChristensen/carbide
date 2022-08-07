use cocoa::appkit::NSEventModifierFlags;
use cocoa::base::{id, nil, selector};
use carbide_core::event::{CustomEvent, EventSink, HasEventSink, HotKey};
use crate::id::Id;
use cocoa::appkit::NSMenuItem as InnerNSMenuItem;
use cocoa::foundation::{NSAutoreleasePool, NSInteger};
use lazy_static::lazy_static;
use objc::runtime::{BOOL, Class, Object, Sel};
use objc::runtime::{NO, YES};
use objc::{msg_send, class, sel, sel_impl};
use crate::menu::NSMenu;
use crate::string::NSString;
use objc::declare::ClassDecl;
use std::ffi::c_void;
use std::sync::mpsc::{channel, Receiver, Sender};
use carbide_core::environment::Environment;

pub struct NSMenuItem {
    pub id: id,
    pub responder: id,
}

impl NSMenuItem {
    pub fn new(title: &str, hot_key: Option<HotKey>) -> NSMenuItem {
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
            id,
            responder: nil
        }
    }

    pub fn separator() -> NSMenuItem {
        let id: id = unsafe { msg_send![class!(NSMenuItem), separatorItem] };

        NSMenuItem {
            id,
            responder: nil,
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

    pub fn set_action(mut self, action: impl Fn(&mut Environment) + 'static, env: &mut Environment) -> NSMenuItem {

        let (pointer, receiver) = CarbideChannel::new(env);

        env.start_stream(receiver, move |_: (), env: &mut Environment| -> bool {
            action(env);
            false
        });

        let responder: id = unsafe { msg_send![CARBIDE_MENU_RESPONDER.0, new] };
        let () = unsafe { msg_send![responder, setHandle: pointer] };
        self.set_tag(42);

        let () = unsafe { msg_send![self.id, setAction: sel!(handle:)] };
        let () = unsafe { msg_send![self.id, setTarget: responder] };

        self
    }

    pub fn set_tag(&mut self, tag: i64) {
        let () = unsafe { msg_send![self.id(), setTag: tag] };
    }

    pub fn tag(&self) -> i64 {
        let tag: NSInteger = unsafe { msg_send![self.id(), tag] };
        tag
    }

    pub fn target(&self) -> id {
        let target: id = unsafe { msg_send![self.id(), target] };
        target
    }

    pub fn has_submenu(&self) -> bool {
        unsafe {
            let b: BOOL = msg_send![self.id(), hasSubmenu];
            b == YES
        }
    }

    pub fn submenu(&self) -> Option<NSMenu> {
        if self.has_submenu() {
            let i: id = unsafe {msg_send![self.id(), submenu]};
            Some(NSMenu { id: i})
        } else {
            None
        }
    }

    pub fn cleanup(&self) {
        let tag = self.tag();
        if tag == 42 {
            let target = self.target();
            unsafe {
                let method: *mut c_void = *(target.as_ref().unwrap()).get_ivar("handle");
                let f = std::mem::transmute::<*mut c_void, *mut CarbideChannel>(method);
                let channel = CarbideChannel::from_raw(f);
                drop(channel)
            }
        }

        if let Some(submenu) = self.submenu() {
            submenu.cleanup()
        }
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

struct NSCarbideResponder(*const Class);

unsafe impl Send for NSCarbideResponder {}

unsafe impl Sync for NSCarbideResponder {}

struct CarbideChannel(Sender<()>, Box<dyn EventSink>);

impl CarbideChannel {
    fn new(sink: &dyn HasEventSink) -> (*const CarbideChannel, Receiver<()>) {
        let (sender, rec) = channel();

        let p = unsafe {
            let mut receiver = Box::into_raw(Box::new(CarbideChannel(sender, sink.event_sink())));
            receiver
        };

        (p, rec)
    }

    unsafe fn from_raw(p: *mut CarbideChannel) -> Box<CarbideChannel> {
        Box::from_raw(p)
    }

    fn received(&self) {
        self.0.send(()).unwrap();
        self.1.call(CustomEvent::AsyncStream);
    }
}

lazy_static! {
    static ref CARBIDE_MENU_RESPONDER: NSCarbideResponder = unsafe {
        let superclass = class!(NSResponder);
        let mut decl = ClassDecl::new("CarbideStreamResponder", superclass).unwrap();
        decl.add_ivar::<*mut c_void>("handle");

        decl.add_class_method(sel!(new), new as extern "C" fn(&Class, Sel) -> id);
        decl.add_method(sel!(setHandle:), set_handle as extern "C" fn(&mut Object, Sel, *mut c_void));
        decl.add_method(sel!(dealloc), dealloc as extern "C" fn(&Object, Sel));

        decl.add_method(
            sel!(handle:),
            handle as extern "C" fn(&Object, Sel, id),
        );

        extern "C" fn handle(this: &Object, _: Sel, object: id) {
            object.print_description();

            unsafe {
                let method: *mut c_void = *this.get_ivar("handle");
                let f = std::mem::transmute::<*mut c_void, *mut CarbideChannel>(method);
                (*f).received();
            }
        }

        extern "C" fn new(class: &Class, _: Sel) -> id {
            unsafe {
                let this: id = msg_send![class, alloc];
                let this: id = msg_send![this, init];
                this
            }
        }

        extern "C" fn set_handle(this: &mut Object, _: Sel, f: *mut c_void) {
            unsafe {
                (*this).set_ivar("handle", f);
            }
        }

        extern "C" fn dealloc(_this: &Object, _: Sel) {
            println!("Dealloc called");
        }

        NSCarbideResponder(decl.register())
    };
}