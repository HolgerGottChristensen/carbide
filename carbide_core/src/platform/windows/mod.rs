#![allow(unsafe_code)]
use crate::environment::Environment;
use crate::dialog::open_dialog::OpenDialog;
use oneshot::Receiver;
use std::ffi::{OsString, OsStr};
use crate::asynchronous::thread_task::ThreadTask;
use windows::Win32::UI::Shell::{IFileOpenDialog, FileOpenDialog, IShellItemArray, IShellItem, SIGDN_FILESYSPATH, FOS_ALLOWMULTISELECT, FOS_PICKFOLDERS, FOS_FORCESHOWHIDDEN, SHCreateItemFromParsingName, IFileSaveDialog, FileSaveDialog};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER, CoTaskMemFree};
use windows::Win32::Foundation::{HWND, PWSTR};
use windows::Win32::UI::Shell::Common::COMDLG_FILTERSPEC;
use std::os::windows::ffi::OsStrExt;
use windows::core::Interface;
use crate::dialog::save_dialog::SaveDialog;

unsafe fn read_to_string(ptr: PWSTR) -> String {
    let mut len = 0usize;
    let mut cursor = ptr;
    loop {
        let val = cursor.0.read();
        if val == 0 {
            break;
        }
        len += 1;
        cursor = PWSTR(cursor.0.add(1));
    }

    let slice = std::slice::from_raw_parts(ptr.0, len);
    String::from_utf16(slice).unwrap()
}

pub fn open_save_panel(env: &Environment, dialog: SaveDialog) -> Receiver<Option<OsString>> {
    let (sender, receiver) = oneshot::channel::<Option<OsString>>();

    let handle = env.hwnd() as isize;

    ThreadTask::new(move || {
        unsafe {
            // Create dialog instance from the GUID with the specific context.
            let save_dialog: IFileSaveDialog = CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER).unwrap();
            match save_dialog.Show(HWND(handle)) {
                Ok(_) => {
                    let items: IShellItem = save_dialog.GetResult().unwrap();
                    let display_name: PWSTR = items.GetDisplayName(SIGDN_FILESYSPATH).unwrap();

                    let filename = read_to_string(display_name);

                    CoTaskMemFree(display_name.0 as _);

                    let path = OsString::from(filename);

                    sender.send(Some(path)).unwrap()
                },
                Err(_) => {
                    sender.send(None).unwrap()
                }
            }
        }
    });

    receiver
}

pub fn open_open_panel(env: &Environment, dialog: OpenDialog) -> Receiver<Option<Vec<OsString>>> {
    let (sender, receiver) = oneshot::channel::<Option<Vec<OsString>>>();

    let handle = env.hwnd() as isize;

    ThreadTask::new(move || {
        unsafe {
            // Create dialog instance from the GUID with the specific context.
            let open_dialog: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER).unwrap();

            if dialog.allow_select_multiple() {
                open_dialog.SetOptions(FOS_ALLOWMULTISELECT.0 as _).unwrap();
            }

            if dialog.allow_select_directories() {
                open_dialog.SetOptions(FOS_PICKFOLDERS.0 as _).unwrap();
            }

            if dialog.allow_show_hidden() {
                open_dialog.SetOptions(FOS_FORCESHOWHIDDEN.0 as _).unwrap();
            }

            if let Some(title) = dialog.showing_title() {
                let mut wide_title: Vec<u16> = OsStr::new(title).encode_wide().chain(std::iter::once(0)).collect();
                let pwstr = PWSTR(wide_title.as_mut_ptr());
                open_dialog.SetTitle(pwstr).unwrap();
            }

            if let Some(button) = dialog.showing_default_button_text() {
                let mut wide_button_title: Vec<u16> = OsStr::new(button).encode_wide().chain(std::iter::once(0)).collect();
                let pwstr = PWSTR(wide_button_title.as_mut_ptr());
                open_dialog.SetOkButtonLabel(pwstr).unwrap();
            }

            if let Some(path) = dialog.showing_starting_directory() {
                if let Some(path) = path.to_str() {
                    let mut wide_path: Vec<u16> =
                        OsStr::new(path).encode_wide().chain(Some(0)).collect();

                    unsafe {
                        let mut item: Option<IShellItem> = None;

                        SHCreateItemFromParsingName(
                            PWSTR(wide_path.as_mut_ptr()),
                            None,
                            &IShellItem::IID,
                            &mut item as *mut _ as *mut _,
                        )
                            .ok();

                        if let Some(item) = item {
                            // For some reason SetDefaultFolder(), does not guarantees default path, so we use SetFolder
                            open_dialog.SetFolder(item).unwrap();
                        }
                    }
                }
            }

            let extensions_list = if let Some(default_type) = dialog.containing_default_type() {
                if let Some(first_extension) = default_type.extensions.first() {
                    let mut extension: Vec<u16> = first_extension.encode_utf16().chain(Some(0)).collect();
                    open_dialog.SetDefaultExtension(PWSTR(extension.as_mut_ptr())).unwrap();
                }
                dialog.containing_allowed_types().cloned().map(|mut list| {
                    list.retain(|f| f != default_type);
                    list.insert(0, default_type.clone());
                    list
                }).or(Some(vec![default_type.clone()]))
            } else {
                dialog.containing_allowed_types().cloned()
            };

            if let Some(extensions) = extensions_list {
                let mut f_list = {
                    let mut f_list = Vec::new();

                    for f in extensions.iter() {
                        let name: Vec<u16> = OsStr::new(&f.name).encode_wide().chain(Some(0)).collect();
                        let ext_string = f
                            .extensions
                            .iter()
                            .map(|item| format!("*.{}", item))
                            .collect::<Vec<_>>()
                            .join(";");

                        let ext: Vec<u16> = OsStr::new(&ext_string)
                            .encode_wide()
                            .chain(std::iter::once(0))
                            .collect();

                        f_list.push((name, ext));
                    }
                    f_list
                };

                let spec: Vec<_> = f_list
                    .iter_mut()
                    .map(|(name, ext)| COMDLG_FILTERSPEC {
                        pszName: PWSTR(name.as_mut_ptr()),
                        pszSpec: PWSTR(ext.as_mut_ptr()),
                    })
                    .collect();

                if !spec.is_empty() {
                    open_dialog.SetFileTypes(spec.len() as _, spec.as_ptr()).unwrap();
                }
            }


            match open_dialog.Show(HWND(handle)) {
                Ok(_) => {
                    let items: IShellItemArray = open_dialog.GetResults().unwrap();
                    let count: u32 = items.GetCount().unwrap();

                    let mut paths = Vec::new();

                    for id in 0..count {
                        let res_item: IShellItem = items.GetItemAt(id).unwrap();

                        let display_name: PWSTR = res_item.GetDisplayName(SIGDN_FILESYSPATH).unwrap();

                        let filename = read_to_string(display_name);

                        CoTaskMemFree(display_name.0 as _);

                        let path = OsString::from(filename);
                        paths.push(path);
                    }

                    sender.send(Some(paths)).unwrap()
                },
                Err(_) => {
                    sender.send(None).unwrap()
                }
            }
        }
    });


    receiver
}