#![allow(non_upper_case_globals, clippy::upper_case_acronyms)]
#![allow(unsafe_code)]

use std::ffi::OsString;
use std::path::PathBuf;

use crate::Color;
use block::ConcreteBlock;
use cocoa::appkit::CGFloat;
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSArray, NSAutoreleasePool, NSInteger, NSProcessInfo, NSString, NSURL};
use objc::{class, msg_send, sel, sel_impl};
use oneshot::Receiver;

use crate::dialog::open_dialog::OpenDialogSettings;
use crate::prelude::Environment;
use crate::state::{InnerState, ValueCell};

pub mod color_dialog;
pub mod menu;

pub(crate) type NSModalResponse = NSInteger;

const NSModalResponseOK: NSModalResponse = 1;
const NSModalResponseCancel: NSModalResponse = 0;

pub fn from_nsstring(s: id) -> String {
    unsafe {
        let slice = std::slice::from_raw_parts(s.UTF8String() as *const _, s.len());
        let result = std::str::from_utf8_unchecked(slice);
        result.into()
    }
}

/// Returns a pointer to the nsstring containing the process name


pub fn make_nsstring(s: &str) -> id {
    unsafe { NSString::alloc(nil).init_str(s).autorelease() }
}

pub fn make_nsurl(url: &PathBuf) -> id {
    unsafe {
        NSURL::alloc(nil)
            .initFileURLWithPath_isDirectory_(
                make_nsstring(url.to_str().expect("Could not convert pathbuf to &str")),
                YES,
            )
            .autorelease()
    }
}

pub fn from_ns_color(color: id) -> Color {
    unsafe {
        let space: id = msg_send![class!(NSColorSpace), genericRGBColorSpace];
        let calibrated_color: id = msg_send![color, colorUsingColorSpace: space];
        let red: CGFloat = msg_send![calibrated_color, redComponent];
        let green: CGFloat = msg_send![calibrated_color, greenComponent];
        let blue: CGFloat = msg_send![calibrated_color, blueComponent];
        let alpha: CGFloat = msg_send![calibrated_color, alphaComponent];
        let color = Color::Rgba(red as f32, green as f32, blue as f32, alpha as f32);
        color
    }
}

pub fn get_control_accent_color() -> Color {
    unsafe { from_ns_color(msg_send![class!(NSColor), controlAccentColor]) }
}

pub fn open_save_panel(env: &Environment) -> Receiver<Option<OsString>> {
    todo!()
    /*let (sender, receiver) = oneshot::channel();

    let sender = InnerState::new(ValueCell::new(Some(sender)));

    unsafe {
        let panel: id = msg_send![class!(NSSavePanel), savePanel];

        let block = ConcreteBlock::new(move |response: NSModalResponse| {
            let sender = sender.clone();
            match response {
                NSModalResponseOK => {
                    let url: id = msg_send![panel, URL];
                    let path: id = msg_send![url, path];
                    //let (path, format) = rewritten_path(panel, path, options);
                    let path: OsString = from_nsstring(path).into();
                    /*Some(FileInfo {
                        path: path.into(),
                        format,
                    })*/
                    sender.borrow_mut().take().map(|s| s.send(Some(path)));
                }
                NSModalResponseCancel => {
                    sender.borrow_mut().take().map(|s| s.send(None));
                }
                _ => unreachable!(),
            }
            /*let view = self_clone.nsview.load();
            if let Some(view) = (*view).as_ref() {
                let view_state: *mut c_void = *view.get_ivar("viewState");
                let view_state = &mut *(view_state as *mut ViewState);
                if ty == FileDialogType::Open {
                    (*view_state).handler.open_file(token, url);
                } else if ty == FileDialogType::Save {
                    (*view_state).handler.save_as(token, url);
                }
            }*/
        });
        let block = block.copy();
        let () =
            msg_send![panel, beginSheetModalForWindow: env.ns_window() completionHandler: block];

        receiver
    }*/
}

pub fn open_open_panel(env: &Environment, dialog: OpenDialogSettings) -> Receiver<Option<Vec<OsString>>> {
    todo!()
    /*let (sender, receiver) = oneshot::channel();

    let sender = InnerState::new(ValueCell::new(Some(sender)));

    unsafe {
        let panel: id = msg_send![class!(NSOpenPanel), openPanel];

        if dialog.allow_select_multiple() {
            let () = msg_send![panel, setAllowsMultipleSelection: YES];
        }

        let mut set_type_filter = true;

        if dialog.allow_select_directories() {
            let () = msg_send![panel, setCanChooseDirectories: YES];
            let () = msg_send![panel, setCanChooseFiles: NO];
            set_type_filter = !dialog.allow_packages_as_directories();
        }

        if dialog.allow_show_hidden() {
            let () = msg_send![panel, setShowsHiddenFiles: YES];
        }

        if let Some(title) = dialog.showing_title() {
            let () = msg_send![panel, setTitle: make_nsstring(title)];
        }

        if let Some(text) = dialog.showing_default_button_text() {
            let () = msg_send![panel, setPrompt: make_nsstring(text)];
        }

        if let Some(message) = dialog.showing_message() {
            let () = msg_send![panel, setMessage: make_nsstring(message)];
        } else {
            let () = msg_send![panel, setMessage: make_nsstring("")];
        }

        if let Some(url) = dialog.showing_starting_directory() {
            let () = msg_send![panel, setDirectoryURL: make_nsurl(url)];
        }

        if set_type_filter {
            // If a default type was specified, then we must reorder the allowed types,
            // because there's no way to specify the default type other than having it be first.
            let allowed_types = if let Some(default_type) = dialog.containing_default_type() {
                if let Some(mut allowed_types) =
                    dialog.containing_allowed_types().map(|v| v.clone())
                {
                    allowed_types.retain(|f| f != default_type);
                    allowed_types.insert(0, default_type.clone());
                    Some(allowed_types)
                } else {
                    Some(vec![default_type.clone()])
                }
            } else {
                dialog
                    .containing_allowed_types()
                    .map(|v| Some(v.clone()))
                    .unwrap_or(None)
            };

            let converted_allowed_types = allowed_types.as_ref().map(|t| {
                t.iter()
                    .flat_map(|spec| spec.extensions().iter().map(|s| make_nsstring(s)))
                    .collect::<Vec<_>>()
            });

            let nsarray_allowed_types = converted_allowed_types
                .as_ref()
                .map(|types| NSArray::arrayWithObjects(nil, types.as_slice()));

            if let Some(nsarray) = nsarray_allowed_types {
                let () = msg_send![panel, setAllowedFileTypes: nsarray];
            }
        }

        let block = ConcreteBlock::new(move |response: NSModalResponse| {
            let sender = sender.clone();
            match response {
                NSModalResponseOK => {
                    let urls: id = msg_send![panel, URLs];

                    let count: NSInteger = msg_send![urls, count];

                    let mut paths = vec![];

                    for index in 0..count {
                        let url: id = msg_send![urls, objectAtIndex: index];
                        let path: id = msg_send![url, path];
                        let path: OsString = from_nsstring(path).into();
                        paths.push(path);
                    }

                    //let (path, format) = rewritten_path(panel, path, options);

                    /*Some(FileInfo {
                        path: path.into(),
                        format,
                    })*/
                    sender.borrow_mut().take().map(|s| s.send(Some(paths)));
                }
                NSModalResponseCancel => {
                    sender.borrow_mut().take().map(|s| s.send(None));
                }
                _ => unreachable!(),
            }
            /*let view = self_clone.nsview.load();
            if let Some(view) = (*view).as_ref() {
                let view_state: *mut c_void = *view.get_ivar("viewState");
                let view_state = &mut *(view_state as *mut ViewState);
                if ty == FileDialogType::Open {
                    (*view_state).handler.open_file(token, url);
                } else if ty == FileDialogType::Save {
                    (*view_state).handler.save_as(token, url);
                }
            }*/
        });
        let block = block.copy();
        let () =
            msg_send![panel, beginSheetModalForWindow: env.ns_window() completionHandler: block];

        receiver
    }*/
}

pub fn open_emoji_dialog() {
    unsafe {
        let app: id = msg_send![class!(NSApplication), sharedApplication];
        let () = msg_send![app, orderFrontCharacterPalette: nil];
    }
}
