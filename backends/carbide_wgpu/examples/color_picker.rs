use std::cell::RefCell;
use std::ffi::c_void;

use cocoa::appkit::{CGFloat, NSColor, NSView};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSPoint, NSRect, NSSize};
use objc::{class, msg_send, sel, sel_impl};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use winit::dpi::PhysicalSize;
use winit::dpi::Size::Physical;
use winit::event::Event;
use winit::event_loop::EventLoop;
use winit::platform::macos::{ActivationPolicy, WindowExtMacOS};
use winit::window::WindowBuilder;

use carbide_core::Color;

fn main() {
    unsafe {
        let mut event_loop = EventLoop::new();

        let inner_window = WindowBuilder::new()
            .with_inner_size(Physical(PhysicalSize::new(600, 600)))
            .with_visible(true)
            .with_title("Title")
            .build(&event_loop).expect("Could not build window");

        let superclass = class!(NSResponder);
        let mut decl = ClassDecl::new("Testername", superclass).unwrap();

        decl.add_class_method(sel!(new), new as extern "C" fn(&Class, Sel) -> id);
        decl.add_method(sel!(dealloc), dealloc as extern "C" fn(&Object, Sel));

        decl.add_method(
            sel!(colorUpdate:),
            did_finish_launching as extern "C" fn(&Object, Sel, id),
        );
        //decl.add_ivar::<*mut c_void>("Tester");

        extern "C" fn did_finish_launching(this: &Object, _: Sel, object: id) {
            unsafe {
                let color: id = msg_send![object, color];
                let red: CGFloat = msg_send![color, redComponent];
                let green: CGFloat = msg_send![color, greenComponent];
                let blue: CGFloat = msg_send![color, blueComponent];
                let alpha: CGFloat = msg_send![color, alphaComponent];
                let color = Color::Rgba(red as f32, green as f32, blue as f32, alpha as f32);
                println!("{:?}", color);
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

        extern "C" fn dealloc(this: &Object, _: Sel) {
            unsafe {
                //let state_ptr: *mut c_void = *(this.get_ivar(AUX_DELEGATE_STATE_NAME));
                // As soon as the box is constructed it is immediately dropped, releasing the underlying
                // memory
                //Box::from_raw(state_ptr as *mut RefCell<AuxDelegateState>);
            }
        }

        let a = decl.register();

        let responder: id = msg_send![a, new];

        let panel: id = msg_send![class!(NSColorPanel), sharedColorPanel];
        let () = msg_send![panel, setContinuous: NO];
        let () = msg_send![panel, setAction: sel!(colorUpdate:)];
        let () = msg_send![panel, setTarget: responder];
        let () = msg_send![panel, setShowsAlpha: YES];
        let () = msg_send![panel, orderFront: inner_window.ns_window()];


        //let panel: id = msg_send![class!(NSFontPanel), sharedFontPanel];
        //let () = msg_send![panel, orderFront: inner_window.ns_window()];

        //let app: id = msg_send![class!(NSApplication), sharedApplication];
        //let () = msg_send![app, orderFrontCharacterPalette: nil];
        //let () = msg_send![app, orderFrontColorPanel: nil];
        //let () = msg_send![app, orderFrontStandardAboutPanel: nil];

        event_loop.run(
            move |event, _, control_flow| {
                match event {
                    Event::NewEvents(_) => {}
                    Event::WindowEvent { window_id, event } => {
                        println!("{:?}", event);
                    }
                    Event::DeviceEvent { .. } => {}
                    Event::UserEvent(_) => {}
                    Event::Suspended => {}
                    Event::Resumed => {}
                    Event::MainEventsCleared => {}
                    Event::RedrawRequested(_) => {}
                    Event::RedrawEventsCleared => {}
                    Event::LoopDestroyed => {}
                }
            });
    }
}