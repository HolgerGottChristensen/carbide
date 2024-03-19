use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use block::ConcreteBlock;
use cocoa::base::id;
use cocoa::base::NO;
use cocoa::base::YES;
use objc::class;
use objc::msg_send;
use objc::sel;
use objc::sel_impl;
use oneshot::Receiver;
use raw_window_handle::{AppKitWindowHandle, HasRawWindowHandle, RawWindowHandle};


use crate::array::NSArray;
use crate::dialogs::NSModalResponse;
use crate::dialogs::NSModalResponseCancel;
use crate::dialogs::NSModalResponseOK;
use crate::FileSpecification;
use crate::id::Id;
use crate::string::NSString;
use crate::url::NSURL;

/// A rust wrapper around: https://developer.apple.com/documentation/appkit/nssavepanel?language=objc
pub struct SavePanel {
    id: id,
}

impl SavePanel {

    /// Creates a new Open panel and initializes it with a default configuration.
    pub fn new() -> SavePanel {
        let panel: id = unsafe {msg_send![class!(NSSavePanel), savePanel]};
        SavePanel {
            id: panel,
        }
    }

    pub fn set_shows_hidden_files(self, show: bool) -> Self {
        unsafe {
            let show = if show {YES} else {NO};
            let () = msg_send![self.id, setShowsHiddenFiles: show];
        }
        self
    }

    pub fn set_resolve_aliases(self, resolve: bool) -> Self {
        unsafe {
            let resolve = if resolve {YES} else {NO};
            let () = msg_send![self.id, setResolveAliases: resolve];
        }
        self
    }

    pub fn set_title(self, title: &str) -> Self {
        let ns_string = NSString::from(title);
        unsafe {
            let () = msg_send![self.id, setTitle: ns_string.id()];
        }
        self
    }

    pub fn set_prompt(self, prompt: &str) -> Self {
        let ns_string = NSString::from(prompt);
        unsafe {
            let () = msg_send![self.id, setPrompt: ns_string.id()];
        }
        self
    }

    pub fn set_message(self, message: &str) -> Self {
        let ns_string = NSString::from(message);
        unsafe {
            let () = msg_send![self.id, setMessage: ns_string.id()];
        }
        self
    }

    /// Sets the start directory
    pub fn set_directory_url(self, url: impl AsRef<Path>) -> Self {
        let url = NSURL::from(url.as_ref());
        unsafe {
            let () = msg_send![self.id, setDirectoryURL: url.id()];
        }
        self
    }

    pub fn set_name_field_label(self, label: &str) -> Self {
        let ns_string = NSString::from(label);
        unsafe {
            let () = msg_send![self.id, setNameFieldLabel: ns_string.id()];
        }
        self
    }

    pub fn set_name_field_string_value(self, string_value: &str) -> Self {
        let ns_string = NSString::from(string_value);
        unsafe {
            let () = msg_send![self.id, nameFieldStringValue: ns_string.id()];
        }
        self
    }

    pub fn set_can_create_directories(self, can: bool) -> Self {
        unsafe {
            let can = if can {YES} else {NO};
            let () = msg_send![self.id, setCanCreateDirectories: can];
        }
        self
    }

    pub fn set_can_select_hidden_extension(self, can: bool) -> Self {
        unsafe {
            let can = if can {YES} else {NO};
            let () = msg_send![self.id, setCanSelectHiddenExtension: can];
        }
        self
    }

    /// The default content type will be the first in the list of allowed files
    pub fn set_allowed_content_types(self, allowed: &Vec<FileSpecification>) -> Self {
        debug_assert!(allowed.len() > 0);

        let extensions = allowed.iter()
            .flat_map(|spec| spec.extensions().iter().map(|s| NSString::from(s.to_string()).id()))
            .collect::<Vec<_>>();

        let ns_array = NSArray::new(&extensions);

        unsafe {
            let () = msg_send![self.id, setAllowedFileTypes: ns_array.id()];
            // The doc says setAllowedContentTypes, but it only works with setAllowedFileTypes
            //let () = msg_send![self.id, setAllowedContentTypes: ns_array.id()];
        }

        self
    }

    pub fn set_allows_other_file_types(self, allow: bool) -> Self {
        unsafe {
            let allow = if allow {YES} else {NO};
            let () = msg_send![self.id, setAllowsOtherFileTypes: allow];
        }
        self
    }

    pub fn set_treats_file_packages_as_directories(self, treat: bool) -> Self {
        unsafe {
            let treat = if treat {YES} else {NO};
            let () = msg_send![self.id, treatsFilePackagesAsDirectories: treat];
        }
        self
    }

    pub fn begin_sheet_modal_for_window(self, window: &impl HasRawWindowHandle) -> Receiver<Option<PathBuf>> {
        let (sender, receiver) = oneshot::channel();

        let sender = Rc::new(RefCell::new(Some(sender)));

        let block = ConcreteBlock::new(move |response: NSModalResponse| {
            let sender = sender.clone();
            #[allow(non_upper_case_globals)]
            match response {
                NSModalResponseOK => {
                    let url: NSURL = NSURL(unsafe {msg_send![self.id, URL]});
                    let path: NSString = NSString(unsafe {msg_send![url.id(), path]});

                    let path: String = path.into();
                    let path = PathBuf::from(path);

                    sender.borrow_mut().take().map(|s| s.send(Some(path)));
                }
                NSModalResponseCancel => {
                    sender.borrow_mut().take().map(|s| s.send(None));
                }
                _ => unreachable!(),
            }
        });

        let block = block.copy();
        let handle = match window.raw_window_handle() {
            RawWindowHandle::AppKit(AppKitWindowHandle { ns_window, .. }) => {
                ns_window
            }
            _ => unreachable!("This is macos platform code, but you have a window that is not AppKit? Please report a bug")
        };
        unsafe {
            let () =
                msg_send![self.id, beginSheetModalForWindow: handle completionHandler: block];
        }


        receiver
    }
}