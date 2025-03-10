use crate::file_type::FileType;
use crate::save_dialog::style::SaveDialogStyle;
use block2::RcBlock;
use carbide::asynchronous::AsyncContext;
use carbide::environment::Environment;
use carbide::SpawnTask;
use carbide_winit::WindowHandleKey;
use objc2::__framework_prelude::Retained;
use objc2::rc::RetainedFromIterator;
use objc2::{msg_send, msg_send_id};
use objc2_app_kit::{NSModalResponse, NSModalResponseCancel, NSModalResponseOK, NSSavePanel, NSWindow};
use objc2_foundation::{MainThreadMarker, NSArray, NSString, NSURL};
use objc2_uniform_type_identifiers::UTType;
use oneshot::RecvError;
use raw_window_handle::{AppKitWindowHandle, RawWindowHandle};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
pub struct MacOSNativeSaveDialogStyle;

impl SaveDialogStyle for MacOSNativeSaveDialogStyle {
    fn open(&self, title: Option<String>, message: Option<String>, prompt: Option<String>, default_file_name: Option<String>, show_hidden_files: bool, path: Option<PathBuf>, file_types: &[FileType], f: Box<dyn Fn(Result<Option<PathBuf>, RecvError>, &mut AsyncContext) + 'static>, env: &mut Environment) {
        let main_thread_marker = MainThreadMarker::new().expect("to be called on the main thread");

        // Create a new open panel with default settings:
        // https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/FileSystemProgrammingGuide/UsingtheOpenandSavePanels/UsingtheOpenandSavePanels.html#//apple_ref/doc/uid/TP40010672-CH4-SW3
        let panel = unsafe { NSSavePanel::savePanel(main_thread_marker) };

        // Set the title of the dialog
        if let Some(title) = title {
            let title = NSString::from_str(&title);
            unsafe { panel.setTitle(Some(&title)) };
        }

        // Set the message of the dialog
        if let Some(message) = message {
            let message = NSString::from_str(&message);
            unsafe { panel.setMessage(Some(&message)) };
        }

        // Set the prompt of the dialog
        if let Some(prompt) = prompt {
            let prompt = NSString::from_str(&prompt);
            unsafe { panel.setPrompt(Some(&prompt)) };
        }

        // Set the show hidden files mode of the panel
        unsafe { panel.setShowsHiddenFiles(show_hidden_files) };

        // Set the URL
        if let Some(path) = path {
            let path = path.to_str().expect("Could not convert pathbuf to &str");
            let path = NSString::from_str(path);
            let path = unsafe { NSURL::fileURLWithPath(&path) };
            unsafe { panel.setDirectoryURL(Some(&path)) };
        }

        // Set the default file name
        if let Some(default_file_name) = default_file_name {
            let default_file_name = NSString::from_str(&default_file_name);
            unsafe { panel.setNameFieldStringValue(&default_file_name) };
        }

        // Set the defaults
        unsafe { panel.setCanCreateDirectories(true) };
        unsafe { panel.setShowsTagField(false) };
        unsafe { panel.setCanSelectHiddenExtension(true) };
        unsafe { panel.setTreatsFilePackagesAsDirectories(false) };

        // Set the allowed filetypes if non-empty
        if !file_types.is_empty() {
            let types_ier = file_types.into_iter()
                .flat_map(|a| a.extensions())
                .map(|a| NSString::from_str(*a))
                .filter_map(|extension| unsafe { UTType::typeWithFilenameExtension(&*extension) });

            let types = NSArray::id_from_iter(types_ier);

            setAllowedContentTypes(&*panel, &*types);
        }


        // Create callback
        let (sender, receiver) = oneshot::channel();

        let sender = Rc::new(RefCell::new(Some(sender)));

        let inner = panel.clone();

        let block = RcBlock::new(move |response: NSModalResponse| {
            let sender = sender.clone();
            match response {
                x if x == NSModalResponseOK => unsafe {
                    let url = inner.URL();

                    let path = url.map(|url| PathBuf::from(url.path().unwrap().to_string()));

                    sender.borrow_mut().take().map(|s| s.send(path));
                },
                x if x == NSModalResponseCancel => {
                    sender.borrow_mut().take().map(|s| s.send(None));
                }
                _ => unimplemented!("Unknown response type: {}", response)
            }
        });

        // Open the panel
        if let Some(&RawWindowHandle::AppKit(AppKitWindowHandle { ns_window, ..})) = env.get::<WindowHandleKey>() {
            let window = unsafe { Retained::retain_autoreleased(ns_window as *mut NSWindow).unwrap() };

            unsafe { panel.beginSheetModalForWindow_completionHandler(&window, &block) };
        } else {
            // Open in a standalone window.
            unsafe { panel.beginWithCompletionHandler(&block)};
        }

        // Spawn listening task for the panel
        receiver.spawn(f)
    }
}

fn allowedContentTypes(panel: &NSSavePanel) -> Retained<NSArray<UTType>> {
    unsafe { msg_send_id![panel, allowedContentTypes] }
}

fn setAllowedContentTypes(panel: &NSSavePanel, allowed_content_types: &NSArray<UTType>) {
    unsafe { msg_send![panel, setAllowedContentTypes: allowed_content_types] }
}