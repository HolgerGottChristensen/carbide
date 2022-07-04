use std::ffi::c_void;
use std::sync::mpsc::Sender;

use cocoa::appkit::CGFloat;
use cocoa::base::{id, NO, YES};
use lazy_static::lazy_static;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};

use crate::dialog::color_dialog::ColorDialog;
use crate::prelude::Environment;
use crate::Color;

struct ColorPickerResponder(*const Class);

unsafe impl Send for ColorPickerResponder {}

unsafe impl Sync for ColorPickerResponder {}

struct ColorPickerChannel(Sender<Color>);

impl ColorPickerChannel {
    fn received(&self, color: Color) {
        self.0.send(color).unwrap();
    }
}

lazy_static! {
    static ref COLOR_PICKER_RESPONDER: ColorPickerResponder = unsafe {
        let superclass = class!(NSResponder);
        let mut decl = ClassDecl::new("Testername", superclass).unwrap();
        decl.add_ivar::<*mut c_void>("method");

        decl.add_class_method(sel!(new), new as extern "C" fn(&Class, Sel) -> id);
        decl.add_method(sel!(setFunctionPointer:), set_function_pointer as extern "C" fn(&mut Object, Sel, *mut c_void));
        decl.add_method(sel!(dealloc), dealloc as extern "C" fn(&Object, Sel));

        decl.add_method(
            sel!(colorUpdate:),
            color_updated as extern "C" fn(&Object, Sel, id),
        );
        //decl.add_ivar::<*mut c_void>("Tester");

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
                /*(*this).set_ivar(
                    "Tester",
                    Box::into_raw(Box::new(RefCell::new(AuxDelegateState {
                        activation_policy: ActivationPolicy::Regular,
                        create_default_menu: true,
                    }))) as *mut c_void,
                );*/
                this
            }
        }

        extern "C" fn set_function_pointer(this: &mut Object, _: Sel, f: *mut c_void) {
            unsafe {
                (*this).set_ivar("method", f);
                //let state_ptr: *mut c_void = *(this.get_ivar(AUX_DELEGATE_STATE_NAME));
                // As soon as the box is constructed it is immediately dropped, releasing the underlying
                // memory
                //Box::from_raw(state_ptr as *mut RefCell<AuxDelegateState>);
            }
        }

        extern "C" fn dealloc(_this: &Object, _: Sel) {
            println!("Dealloc called");
            //unsafe {
                //let state_ptr: *mut c_void = *(this.get_ivar(AUX_DELEGATE_STATE_NAME));
                // As soon as the box is constructed it is immediately dropped, releasing the underlying
                // memory
                //Box::from_raw(state_ptr as *mut RefCell<AuxDelegateState>);
            //}
        }

        ColorPickerResponder(decl.register())
    };
}

pub fn open_color_dialog(
    env: &Environment,
    sender: std::sync::mpsc::Sender<Color>,
    dialog: ColorDialog,
) {
    let inner_window = env.ns_window();

    unsafe {
        let mut receiver = Box::pin(ColorPickerChannel(sender));

        let responder: id = msg_send![COLOR_PICKER_RESPONDER.0, new];
        let () = msg_send![responder, setFunctionPointer: receiver.as_mut().get_unchecked_mut()];

        // This is very much a possible leak, because we never clean it up, and we are
        // probably not guarantied that the memory is not overridden.
        // It seems to work for now though.
        std::mem::forget(receiver);

        let panel: id = msg_send![class!(NSColorPanel), sharedColorPanel];
        let () = msg_send![panel, setAction: sel!(colorUpdate:)];
        let () = msg_send![panel, setTarget: responder];

        /*if let Some(color) = dialog.initial_color {
            let rgba = color.to_rgb();
            println!("{:?}", rgba);
            let color: id = msg_send![class!(NSColor), colorWithRed: rgba.0 green: rgba.1 blue: rgba.2 alpha: rgba.3];
            let () = msg_send![panel, setColor: color];
        }*/

        if dialog.show_alpha {
            let () = msg_send![panel, setShowsAlpha: YES];
        } else {
            let () = msg_send![panel, setShowsAlpha: NO];
        }

        if !dialog.continuous {
            let () = msg_send![panel, setContinuous: NO];
        }

        let () = msg_send![panel, orderFront: inner_window];
    }
}
