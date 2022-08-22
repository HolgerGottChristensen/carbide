use std::ffi::c_void;
use std::sync::mpsc::{Sender, Receiver, channel};

use cocoa::appkit::CGFloat;
use cocoa::base::{id, NO, YES};
use lazy_static::lazy_static;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};
use raw_window_handle::{AppKitHandle, HasRawWindowHandle, RawWindowHandle};
use carbide_core::Color;
use carbide_core::event::{CustomEvent, EventSink, HasEventSink, HasRawWindowHandleAndEventSink};
use carbide_core::environment::Environment;
use crate::string::NSString;


pub struct ColorPanel {
    id: id,
}

impl ColorPanel {
    pub fn new() -> ColorPanel {
        let panel: id = unsafe {msg_send![class!(NSColorPanel), sharedColorPanel]};

        ColorPanel {
            id: panel
        }
    }

    pub fn set_continuous(mut self, continuous: bool) -> Self {
        unsafe {
            let continuous = if continuous {YES} else {NO};
            let () = msg_send![self.id, setContinuous: continuous];
        }
        self
    }

    pub fn set_shows_alpha(mut self, shows_alpha: bool) -> Self {
        unsafe {
            let shows_alpha = if shows_alpha {YES} else {NO};
            let () = msg_send![self.id, setShowsAlpha: shows_alpha];
        }
        self
    }

    pub fn order_front(self, window: &impl HasRawWindowHandleAndEventSink) -> Receiver<Color> {
        let (sender, rec) = channel();

        unsafe {
            let mut receiver = Box::pin(ColorPickerChannel(sender, window.event_sink()));

            let responder: id = msg_send![COLOR_PICKER_RESPONDER.0, new];
            let () = msg_send![responder, setFunctionPointer: receiver.as_mut().get_unchecked_mut()];

            // This is very much a possible leak, because we never clean it up, and we are
            // probably not guarantied that the memory is not overridden.
            // It seems to work for now though.
            std::mem::forget(receiver);

            let () = msg_send![self.id, setAction: sel!(colorUpdate:)];
            let () = msg_send![self.id, setTarget: responder];

            let handle = match window.raw_window_handle() {
                RawWindowHandle::AppKit(AppKitHandle { ns_window, .. }) => {
                    ns_window
                }
                _ => unreachable!("This is macos platform code, but you have a window that is not AppKit? Please report a bug")
            };

            let () = msg_send![self.id, orderFront: handle];
        }

        rec
    }
}


struct ColorPickerChannel(Sender<Color>, Box<dyn EventSink>);

impl ColorPickerChannel {
    fn received(&self, color: Color) {
        self.0.send(color).unwrap();
        self.1.call(CustomEvent::AsyncStream);
    }
}






struct ColorPickerResponder(*const Class);

unsafe impl Send for ColorPickerResponder {}

unsafe impl Sync for ColorPickerResponder {}

lazy_static! {
    static ref COLOR_PICKER_RESPONDER: ColorPickerResponder = unsafe {
        let superclass = class!(NSResponder);
        let mut decl = ClassDecl::new("ColorPickerResponder", superclass).unwrap();
        decl.add_ivar::<*mut c_void>("method");

        decl.add_class_method(sel!(new), new as extern "C" fn(&Class, Sel) -> id);
        decl.add_method(sel!(setFunctionPointer:), set_function_pointer as extern "C" fn(&mut Object, Sel, *mut c_void));
        decl.add_method(sel!(dealloc), dealloc as extern "C" fn(&Object, Sel));

        decl.add_method(
            sel!(colorUpdate:),
            color_updated as extern "C" fn(&Object, Sel, id),
        );

        extern "C" fn color_updated(this: &Object, _: Sel, object: id) {
            unsafe {
                let color: id = msg_send![object, color];
                let red: CGFloat = msg_send![color, redComponent];
                let green: CGFloat = msg_send![color, greenComponent];
                let blue: CGFloat = msg_send![color, blueComponent];
                let alpha: CGFloat = msg_send![color, alphaComponent];
                let color = Color::Rgba(red as f32, green as f32, blue as f32, alpha as f32);
                let method: *mut c_void = *this.get_ivar("method");
                let f = std::mem::transmute::<*mut c_void, *mut ColorPickerChannel>(method);
                (*f).received(color);
            }
        }

        extern "C" fn new(class: &Class, _: Sel) -> id {
            unsafe {
                let this: id = msg_send![class, alloc];
                let this: id = msg_send![this, init];
                this
            }
        }

        extern "C" fn set_function_pointer(this: &mut Object, _: Sel, f: *mut c_void) {
            unsafe {
                (*this).set_ivar("method", f);
            }
        }

        extern "C" fn dealloc(_this: &Object, _: Sel) {
            println!("Dealloc called");
        }

        ColorPickerResponder(decl.register())
    };
}