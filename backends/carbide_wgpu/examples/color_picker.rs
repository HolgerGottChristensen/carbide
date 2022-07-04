use std::cell::RefCell;
use std::ffi::c_void;
use std::marker::PhantomPinned;
use std::sync::Mutex;

use cocoa::appkit::NSWindowTitleVisibility::NSWindowTitleHidden;
use cocoa::appkit::{CGFloat, NSApplication, NSColor, NSMenu, NSMenuItem, NSView, NSWindow};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel, BOOL};
use objc::{class, msg_send, sel, sel_impl};
use winit::dpi::PhysicalSize;
use winit::dpi::Size::Physical;
use winit::event::{Event, ModifiersState, WindowEvent};
use winit::event_loop::EventLoop;
use winit::platform::macos::{ActivationPolicy, WindowBuilderExtMacOS, WindowExtMacOS};
use winit::window::WindowBuilder;

use carbide_core::event::{HotKey, Key, ModifierKey};
use carbide_core::platform::mac::make_nsstring;
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

        //let app: id = msg_send![class!(NSApplication), sharedApplication];
        //let window: id = msg_send![app, mainWindow];
        //let () = msg_send![window, setAlphaValue: 0.3];
        //inner_window.set_title("true");
        //let title = NSString::alloc(nil).init_str("TitleNew");
        //window.setTitle_(title);
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

        /*let app: id = msg_send![class!(NSApplication), sharedApplication];

        let title = make_nsstring("Hello");
        let menu = NSMenu::alloc(nil).initWithTitle_(title).autorelease();
        //let menu_item = make_menu_item("Item 1", None, "".to_string());
        let () = msg_send![menu, setAutoenablesItems: NO];
        //let () = msg_send![app, setHelpMenu: menu];
        //let help: id = msg_send![app, windowsMenu];
        //println!("{:?}", help);

        let upper_menu_item = make_menu_item("Item 1", None, "".to_string());
        let () = msg_send![upper_menu_item, setSubmenu: menu];


        let current_menu: id = msg_send![app, mainMenu];
        let () = msg_send![current_menu, addItem: upper_menu_item];
        //let () = msg_send![help, setSub: menu];*/

        event_loop.run(
            move |event, _, control_flow| {
                match event {
                    Event::NewEvents(_) => {}
                    Event::WindowEvent {
                        event: WindowEvent::ModifiersChanged(m),
                        window_id,
                    } if window_id == inner_window.id() => {
                        if !m.contains(ModifiersState::SHIFT) { return }
                        unsafe {
                            let title = NSString::alloc(nil).init_str("").autorelease();
                            let menu_item_title = NSString::alloc(nil).init_str("Menu item").autorelease();
                            let menu_item_key = NSString::alloc(nil).init_str("\u{0008}").autorelease();


                            let menu = NSMenu::alloc(nil).initWithTitle_(title).autorelease();
                            let () = msg_send![menu, setAutoenablesItems: NO];

                            let menu_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
                                menu_item_title,
                                sel!(test:),
                                menu_item_key,
                            ).autorelease();

                            menu.addItem_(menu_item);

                            let app: id = msg_send![class!(NSApplication), sharedApplication];
                            let window: id = msg_send![app, mainWindow];
                            let view: id = msg_send![window, contentView];

                            let mut location: NSPoint = msg_send![window, mouseLocationOutsideOfEventStream];

                            let _: BOOL = msg_send![menu, popUpMenuPositioningItem: nil atLocation: location inView: view as *const _];
                        }
                    }
                    Event::WindowEvent { window_id, event } => {
                        match event {
                            WindowEvent::Resized(_) => {}
                            WindowEvent::Moved(_) => {}
                            WindowEvent::CloseRequested => {}
                            WindowEvent::Destroyed => {}
                            WindowEvent::DroppedFile(_) => {}
                            WindowEvent::HoveredFile(_) => {}
                            WindowEvent::HoveredFileCancelled => {}
                            WindowEvent::ReceivedCharacter(_) => {}
                            WindowEvent::Focused(focused) => {
                                if focused {
                                    //let () = msg_send![help, addMenu: menu];
                                    //let () = msg_send![help, addItem: menu];
                                    //NSApplication::(app, menu);
                                    //let () = msg_send![app, setValue: menu forKey: ttt];


                                    let app: id = msg_send![class!(NSApplication), sharedApplication];
                                    let window: id = msg_send![app, mainWindow];
                                    //let () = msg_send![window, setAlphaValue: 0.3];
                                    //let toolbar_alloc: id = msg_send![class!(NSToolbar), alloc];
                                    //let toolbar: id = msg_send![toolbar_alloc, init];

                                    let current_menu: id = msg_send![app, mainMenu];


                                    let title = make_nsstring("Edit");
                                    let menu = NSMenu::alloc(nil).initWithTitle_(title).autorelease();
                                    let () = msg_send![menu, setAutoenablesItems: NO];


                                    let menu_item = make_menu_item("Item 1", None, "", true);
                                    let () = msg_send![menu, addItem: menu_item];

                                    let () = msg_send![menu, addItem: make_separator()];

                                    let menu_item = make_menu_item("Item 2", None, "", true);
                                    let () = msg_send![menu, addItem: menu_item];

                                    let () = msg_send![menu, addItem: make_separator()];

                                    let menu_item = make_menu_item("Item 3", None, "", true);
                                    let () = msg_send![menu, addItem: menu_item];

                                    /*let sep: id = msg_send![class!(NSMenuItem), separatorItem];
                                    let () = msg_send![menu, addItem: sep];

                                    for i in 0u8..255 {
                                        let menu_item = make_menu_item("Item", Some(HotKey { key: Key::Return, modifier: ModifierKey::GUI }), String::from(i as char));
                                        let () = msg_send![menu, addItem: menu_item];
                                    }*/


                                    let upper_menu_item = NSMenuItem::alloc(nil).autorelease();
                                    let () = msg_send![upper_menu_item, setSubmenu: menu];
                                    let () = msg_send![current_menu, addItem: upper_menu_item];

                                    //let () = msg_send![app, setServicesMenu: menu];
                                    let () = msg_send![app, setWindowsMenu: menu];
                                    //let () = msg_send![app, setHelpMenu: menu];

                                    let menu_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
                                        make_nsstring("Zoom"),
                                        sel!(zoom:),
                                        make_nsstring(""),
                                    ).autorelease();


                                    let () = msg_send![menu_item, setEnabled: YES];
                                    let () = msg_send![menu_item, setState: NO];

                                    let () = msg_send![menu, insertItem: menu_item atIndex: 0];


                                    //println!("HERE");
                                    //window.setToolbar_(toolbar);
                                    //window.setTitlebarAppearsTransparent_(YES);
                                    //app.setWindowsMenu_(menu);

                                    window.setBackgroundColor_(msg_send![class!(NSColor), systemBlueColor]);

                                    //let () = msg_send![window, toolbar: toolbar];
                                    //let () = msg_send![window, setTitleVisibility: NSWindowTitleHidden];
                                    //inner_window.set_title("true");
                                    //let title = NSString::alloc(nil).init_str("/Users/holgergottchristensen/carbide/backends/carbide_wgpu/examples/color_picker.rs");
                                    //window.setTitleWithRepresentedFilename_(title);
                                }
                            }
                            WindowEvent::KeyboardInput { .. } => {

                            }
                            WindowEvent::ModifiersChanged(_) if window_id == inner_window.id() => {
                                /*let lock = Mutex::new(());
                                let l = lock.lock();
                                let l2 = lock.lock();

                                let n = (l, l2);*/
                                /*if menu_opened { return }
                                println!("Open menu");
                                let title = make_nsstring("");
                                let menu = NSMenu::alloc(nil).initWithTitle_(title).autorelease();
                                let () = msg_send![menu, setAutoenablesItems: NO];

                                let menu_item = unsafe {
                                    let menu_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
                                        make_nsstring("Test"),
                                        sel!(aaa:),
                                        make_nsstring(""),
                                    ).autorelease();

                                    menu_item
                                };
                                menu.addItem_(menu_item);
                                /*let menu_item = make_menu_item("Item 1", None, "".to_string());
                                let () = msg_send![menu, addItem: menu_item];
                                let menu_item = make_menu_item("Item 1", None, "".to_string());
                                let () = msg_send![menu, addItem: menu_item];
                                let menu_item = make_menu_item("Item 1", None, "".to_string());
                                let () = msg_send![menu, addItem: menu_item];
                                let menu_item = make_menu_item("Item 1", None, "".to_string());
                                let () = msg_send![menu, addItem: menu_item];

                                let point = NSPoint::new(CGFloat::from(30), CGFloat::from(30));
*/
                                let app: id = msg_send![class!(NSApplication), sharedApplication];
                                let window: id = msg_send![app, mainWindow];
                                let view: id = msg_send![window, contentView];

                                let mut location: NSPoint = msg_send![window, mouseLocationOutsideOfEventStream];

                                let _: BOOL = msg_send![menu, popUpMenuPositioningItem: nil atLocation: location inView: view as *const _];
                                println!("Opened");
                                menu_opened = true;*/

                            }
                            WindowEvent::CursorMoved { .. } => {}
                            WindowEvent::CursorEntered { .. } => {}
                            WindowEvent::CursorLeft { .. } => {}
                            WindowEvent::MouseWheel { .. } => {}
                            WindowEvent::MouseInput { device_id, state, button, modifiers } => {

                            }
                            WindowEvent::TouchpadPressure { .. } => {}
                            WindowEvent::AxisMotion { .. } => {}
                            WindowEvent::Touch(_) => {}
                            WindowEvent::ScaleFactorChanged { .. } => {}
                            WindowEvent::ThemeChanged(_) => {}
                            _ => {}
                        }
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

fn make_menu_item(title: &str, key: Option<HotKey>, code: &str, enabled: bool) -> id {
    unsafe {
        let key_combination = code; //key.map(HotKey::key_equivalent).unwrap_or("".to_string());

        let menu_item = NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(
                make_nsstring(title),
                sel!(handleMenuItem:),
                make_nsstring(key_combination),
            )
            .autorelease();

        /*if let Some(mask) = key.map(HotKey::key_modifier_mask) {
            let () = msg_send![menu_item, setKeyEquivalentModifierMask: mask];
        }*/

        if enabled {
            let () = msg_send![menu_item, setEnabled: YES];
        } else {
            let () = msg_send![menu_item, setEnabled: NO];
        }

        let () = msg_send![menu_item, setState: NO];
        menu_item
    }
}

fn make_separator() -> id {
    unsafe { msg_send![class!(NSMenuItem), separatorItem] }
}
