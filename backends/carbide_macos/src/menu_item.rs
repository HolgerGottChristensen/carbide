use std::ffi::c_void;
use std::sync::mpsc::{channel, Receiver, Sender};

use cocoa::appkit::NSEventModifierFlags;
use cocoa::appkit::NSMenuItem as InnerNSMenuItem;
use cocoa::base::{id, nil};
use cocoa::foundation::{NSAutoreleasePool, NSInteger};
use lazy_static::lazy_static;
use objc::{class, msg_send, sel, sel_impl};
use objc::declare::ClassDecl;
use objc::runtime::{BOOL, Class, Object, Sel};
use objc::runtime::{NO, YES};

use carbide_core::environment::Environment;
use carbide_core::event::{CustomEvent, EventSink, HasEventSink, HotKey, Key, ModifierKey};
use carbide_core::widget::MenuAction;

use crate::id::Id;
use crate::menu::NSMenu;
use crate::string::NSString;

pub struct NSMenuItem {
    pub id: id,
    pub responder: id,
}

impl NSMenuItem {
    pub fn new(title: &str, hotkey: Option<HotKey>) -> NSMenuItem {

        let k = hotkey.map(KeyEquivalent::from).unwrap_or_else(|| {
            KeyEquivalent {
                key: NSString::from(""),
                masks: None
            }
        });

        let title = NSString::from(title);

        let id = unsafe {
            InnerNSMenuItem::alloc(nil)
                .initWithTitle_action_keyEquivalent_(title.id(), sel!(handleMenuItem:), k.key.id())
                .autorelease()
        };

        if let Some(mask) = k.masks {
            let () = unsafe { msg_send![id, setKeyEquivalentModifierMask: mask] };
        }

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

    pub fn set_enabled(self, enabled: bool) -> NSMenuItem {
        let enabled = if enabled { YES } else { NO };
        unsafe {
            let () = msg_send![self.id(), setEnabled: enabled];
        }
        self
    }

    pub fn set_hidden(self, hidden: bool) -> NSMenuItem {
        let hidden = if hidden { YES } else { NO };
        unsafe {
            let () = msg_send![self.id(), setHidden: hidden];
        }
        self
    }

    pub fn set_submenu(self, menu: NSMenu) -> NSMenuItem {
        unsafe {
            let () = msg_send![self.id(), setSubmenu: menu];
        }
        self
    }

    pub fn set_action(mut self, action: Box<dyn MenuAction>, env: &mut Environment) -> NSMenuItem {

        let (pointer, receiver) = CarbideChannel::new(env);

        env.start_stream(receiver, move |_: (), env: &mut Environment| -> bool {
            (action)(env);
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

struct KeyEquivalent {
    key: NSString,
    masks: Option<NSEventModifierFlags>,
}

impl From<HotKey> for KeyEquivalent {
    fn from(hot: HotKey) -> Self {
        let char = match hot.key {
            Key::Backspace => '\u{0008}',
            Key::Delete => '\u{0008}',
            Key::Return => '\u{0003}',

            Key::Insert => '\u{F727}',
            Key::Home => '\u{F729}',
            Key::End => '\u{F72B}',
            Key::PageUp => '\u{F72C}',
            Key::PageDown => '\u{F72D}',
            Key::PrintScreen => '\u{F72E}',
            Key::ScrollLock => '\u{F72F}',
            Key::Up => '\u{F700}',
            Key::Down => '\u{F701}',
            Key::Left => '\u{F702}',
            Key::Right => '\u{F703}',

            Key::F1 => '\u{F704}',
            Key::F2 => '\u{F705}',
            Key::F3 => '\u{F706}',
            Key::F4 => '\u{F707}',
            Key::F5 => '\u{F708}',
            Key::F6 => '\u{F709}',
            Key::F7 => '\u{F70A}',
            Key::F8 => '\u{F70B}',
            Key::F9 => '\u{F70C}',
            Key::F10 => '\u{F70D}',
            Key::F11 => '\u{F70E}',
            Key::F12 => '\u{F70F}',
            Key::F13 => '\u{F710}',
            Key::F14 => '\u{F711}',
            Key::F15 => '\u{F712}',
            Key::F16 => '\u{F713}',
            Key::F17 => '\u{F714}',
            Key::F18 => '\u{F715}',
            Key::F19 => '\u{F716}',
            Key::F20 => '\u{F717}',
            Key::F21 => '\u{F718}',
            Key::F22 => '\u{F719}',
            Key::F23 => '\u{F71A}',
            Key::F24 => '\u{F71B}',

            Key::Pause => '\u{F730}',
            Key::SysReq => '\u{F731}',

            //NSBreakFunctionKey          = 0xF732,
            //NSResetFunctionKey          = 0xF733,

            Key::Stop => '\u{F734}',
            Key::Menu => '\u{F735}',

            //NSUserFunctionKey           = 0xF736,
            //NSSystemFunctionKey         = 0xF737,
            //NSPrintFunctionKey          = 0xF738,
            //NSClearLineFunctionKey      = 0xF739,
            //NSClearDisplayFunctionKey   = 0xF73A,
            //NSInsertLineFunctionKey     = 0xF73B,
            //NSDeleteLineFunctionKey     = 0xF73C,
            //NSInsertCharFunctionKey     = 0xF73D,
            //NSDeleteCharFunctionKey     = 0xF73E,
            //NSPrevFunctionKey           = 0xF73F,
            //NSNextFunctionKey           = 0xF740,

            Key::Select => '\u{F741}',
            Key::Execute => '\u{F741}',
            Key::Undo => '\u{F743}',

            //NSRedoFunctionKey           = 0xF744,

            Key::Find => '\u{F745}',
            Key::Help => '\u{F746}',
            Key::Mode => '\u{F747}',

            Key::Tab => '\t',
            Key::Space => ' ',
            Key::Exclaim => '!',
            Key::Hash => '#',
            Key::Dollar => '$',
            Key::Percent => '%',
            Key::Ampersand => '&',
            Key::Quote => '"',
            Key::LeftParen => '(',
            Key::RightParen => ')',
            Key::Asterisk => '*',
            Key::Plus => '+',
            Key::Comma => ',',
            Key::Minus => '-',
            Key::Period => '.',
            Key::Slash => '/',
            Key::D0 => '0',
            Key::D1 => '1',
            Key::D2 => '2',
            Key::D3 => '3',
            Key::D4 => '4',
            Key::D5 => '5',
            Key::D6 => '6',
            Key::D7 => '7',
            Key::D8 => '8',
            Key::D9 => '9',
            Key::Colon => ':',
            Key::Semicolon => ';',
            Key::Less => '<',
            Key::Equals => '=',
            Key::Greater => '>',
            Key::Question => '?',
            Key::At => '@',
            Key::LeftBracket => '[',
            Key::Backslash => '\\',
            Key::RightBracket => ']',
            Key::A => 'a',
            Key::B => 'b',
            Key::C => 'c',
            Key::D => 'd',
            Key::E => 'e',
            Key::F => 'f',
            Key::G => 'g',
            Key::H => 'h',
            Key::I => 'i',
            Key::J => 'j',
            Key::K => 'k',
            Key::L => 'l',
            Key::M => 'm',
            Key::N => 'n',
            Key::O => 'o',
            Key::P => 'p',
            Key::Q => 'q',
            Key::R => 'r',
            Key::S => 's',
            Key::T => 't',
            Key::U => 'u',
            Key::V => 'v',
            Key::W => 'w',
            Key::X => 'x',
            Key::Y => 'y',
            Key::Z => 'z',
            _ => panic!("You are trying to use a key that is not mapped on the macos platform."),
        };

        let modifiers = if hot.modifier == ModifierKey::NO_MODIFIER {
            None
        } else {
            let mut k = NSEventModifierFlags::empty();

            if hot.modifier.contains(ModifierKey::SHIFT) {
                k = k | NSEventModifierFlags::NSShiftKeyMask;
            }
            if hot.modifier.contains(ModifierKey::CTRL) {
                k = k | NSEventModifierFlags::NSControlKeyMask;
            }
            if hot.modifier.contains(ModifierKey::ALT) {
                k = k | NSEventModifierFlags::NSAlternateKeyMask;
            }
            if hot.modifier.contains(ModifierKey::GUI) {
                k = k | NSEventModifierFlags::NSCommandKeyMask;
            }

            Some(k)
        };

        KeyEquivalent {
            key: NSString::from(char),
            masks: modifiers
        }
    }
}

struct NSCarbideResponder(*const Class);

unsafe impl Send for NSCarbideResponder {}

unsafe impl Sync for NSCarbideResponder {}

struct CarbideChannel(Sender<()>, Box<dyn EventSink>);

impl CarbideChannel {
    fn new(sink: &dyn HasEventSink) -> (*const CarbideChannel, Receiver<()>) {
        let (sender, rec) = channel();

        let p = unsafe {
            let receiver = Box::into_raw(Box::new(CarbideChannel(sender, sink.event_sink())));
            receiver
        };

        (p, rec)
    }

    unsafe fn from_raw(p: *mut CarbideChannel) -> Box<CarbideChannel> {
        Box::from_raw(p)
    }

    fn received(&self) {
        self.0.send(()).unwrap();
        self.1.send(CustomEvent::AsyncStream);
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