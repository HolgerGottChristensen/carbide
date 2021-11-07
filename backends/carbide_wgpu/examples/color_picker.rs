use std::cell::RefCell;
use std::ffi::c_void;
use std::marker::PhantomPinned;

use cocoa::appkit::{CGFloat, NSColor, NSView, NSWindow};
use cocoa::appkit::NSWindowTitleVisibility::NSWindowTitleHidden;
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSPoint, NSRect, NSSize, NSString};
use objc::{class, msg_send, sel, sel_impl};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use winit::dpi::PhysicalSize;
use winit::dpi::Size::Physical;
use winit::event::Event;
use winit::event_loop::EventLoop;
use winit::platform::macos::{ActivationPolicy, WindowBuilderExtMacOS, WindowExtMacOS};
use winit::window::WindowBuilder;

use carbide_core::Color;

//use carbide_core::platform::mac::color_dialog::{COLOR_PICKER_RESPONDER, ColorPickerChannel};

fn main() {
    unsafe {
        let mut event_loop = EventLoop::new();

        let inner_window = WindowBuilder::new()
            .with_inner_size(Physical(PhysicalSize::new(600, 600)))
            .with_visible(true)
            .with_title("Title")
            //.with_titlebar_transparent(true)
            //.with_fullsize_content_view(true)
            .build(&event_loop)
            .expect("Could not build window");


        let app: id = msg_send![class!(NSApplication), sharedApplication];
        let window: id = msg_send![app, mainWindow];
        //let () = msg_send![window, setAlphaValue: 0.3];
        inner_window.set_title("true");
        let title = NSString::alloc(nil).init_str("TitleNew");
        window.setTitle_(title);
        //let () = msg_send![window, setTitle: title];

        /*fn test(col: Color) {
            println!("Oh shit: {:?}", col);
        }

        let (sender, receiver) = std::sync::mpsc::channel();

        let mut test2 = Box::pin(ColorPickerChannel(sender));

        std::thread::spawn(move || {
            loop {
                let res = receiver.recv();
                println!("Color received: {:?}", res.unwrap());
            }
        });

        let responder: id = msg_send![COLOR_PICKER_RESPONDER.0, new];
        let () = msg_send![responder, setFunctionPointer: test2.as_mut().get_unchecked_mut()];
        //let () = msg_send![responder, setFunctionPointer: test as *mut c_void];

        let panel: id = msg_send![class!(NSColorPanel), sharedColorPanel];
        let () = msg_send![panel, setContinuous: NO];
        let () = msg_send![panel, setAction: sel!(colorUpdate:)];
        let () = msg_send![panel, setTarget: responder];
        let () = msg_send![panel, setShowsAlpha: YES];
        let () = msg_send![panel, orderFront: inner_window.ns_window()];*/


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
                        //let app: id = msg_send![class!(NSApplication), sharedApplication];
                        //let window: id = msg_send![app, mainWindow];
                        //let () = msg_send![window, setAlphaValue: 0.3];
                        //let toolbar_alloc: id = msg_send![class!(NSToolbar), alloc];
                        //let toolbar: id = msg_send![toolbar_alloc, init];
                        //println!("HERE");
                        //window.setToolbar_(toolbar);
                        //window.setTitlebarAppearsTransparent_(YES);
                        //window.setBackgroundColor_(msg_send![class!(NSColor), systemBlueColor]);

                        //let () = msg_send![window, toolbar: toolbar];
                        //let () = msg_send![window, setTitleVisibility: NSWindowTitleHidden];
                        //inner_window.set_title("true");
                        //let title = NSString::alloc(nil).init_str("/Users/holgergottchristensen/carbide/backends/carbide_wgpu/examples/color_picker.rs");
                        //window.setTitleWithRepresentedFilename_(title);
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