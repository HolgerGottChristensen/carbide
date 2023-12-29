// use std::ffi::OsStr;
// use std::os::windows::ffi::OsStrExt;
// use std::path::PathBuf;
// use std::ptr;
// use windows::core::Result as WindowsResult;
// use windows::Win32::Foundation::{HWND, PWSTR};
// use windows::Win32::System::Com::{
//     CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
//     COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
// };
// use windows::Win32::UI::Shell::*;
// use winit::dpi::PhysicalSize;
// use winit::dpi::Size::Physical;
// use winit::event::{Event, WindowEvent};
// use winit::event_loop::EventLoop;
// use winit::platform::windows::WindowExtWindows;
// use winit::window::WindowBuilder;
//
// fn main() {
//     let mut event_loop = EventLoop::new();
//
//     let inner_window = WindowBuilder::new()
//         .with_inner_size(Physical(PhysicalSize::new(600, 600)))
//         .with_visible(true)
//         .with_title("Title")
//         //.with_titlebar_transparent(true)
//         //.with_fullsize_content_view(true)
//         .build(&event_loop)
//         .expect("Could not build window");
//
//     unsafe fn read_to_string(ptr: PWSTR) -> String {
//         let mut len = 0usize;
//         let mut cursor = ptr;
//         loop {
//             let val = cursor.0.read();
//             if val == 0 {
//                 break;
//             }
//             len += 1;
//             cursor = PWSTR(cursor.0.add(1));
//         }
//
//         let slice = std::slice::from_raw_parts(ptr.0, len);
//         String::from_utf16(slice).unwrap()
//     }
//
//     /// Makes sure that COM lib is initialized
//     pub fn init_com<T, F: FnOnce() -> T>(f: F) -> WindowsResult<T> {
//         unsafe {
//             CoInitializeEx(
//                 ptr::null_mut(),
//                 COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
//             )?
//         };
//
//         let out = f();
//
//         unsafe {
//             CoUninitialize();
//         }
//
//         Ok(out)
//     }
//
//     let window_id = inner_window.hwnd() as isize;
//
//     std::thread::spawn(move || {
//         unsafe {
//             init_com(|| {
//                 let dialog: IFileOpenDialog =
//                     CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER).unwrap();
//                 let mut wide_title: Vec<u16> = OsStr::new("Hejsa")
//                     .encode_wide()
//                     .chain(std::iter::once(0))
//                     .collect();
//                 let pwstr = PWSTR(wide_title.as_mut_ptr());
//                 dialog.SetOkButtonLabel(pwstr).unwrap();
//                 dialog.SetTitle(pwstr).unwrap();
//                 //dialog.SetOptions(FOS_FORCESHOWHIDDEN.0 as _);
//                 dialog.Show(HWND(window_id));
//                 let items: IShellItemArray = dialog.GetResults().unwrap();
//                 let count: u32 = items.GetCount().unwrap();
//
//                 let mut paths = Vec::new();
//
//                 for id in 0..count {
//                     let res_item: IShellItem = items.GetItemAt(id).unwrap();
//
//                     let display_name: PWSTR = res_item.GetDisplayName(SIGDN_FILESYSPATH).unwrap();
//
//                     let filename = read_to_string(display_name);
//
//                     CoTaskMemFree(display_name.0 as _);
//
//                     let path = PathBuf::from(filename);
//                     paths.push(path);
//                 }
//                 println!("{:?}", paths);
//             });
//         }
//     });
//
//     event_loop.run(move |event, _, control_flow| match event {
//         Event::NewEvents(_) => {}
//         Event::WindowEvent { window_id, event } => match event {
//             WindowEvent::Resized(_) => {}
//             WindowEvent::Moved(_) => {}
//             WindowEvent::CloseRequested => {}
//             WindowEvent::Destroyed => {}
//             WindowEvent::DroppedFile(_) => {}
//             WindowEvent::HoveredFile(_) => {}
//             WindowEvent::HoveredFileCancelled => {}
//             WindowEvent::ReceivedCharacter(_) => {}
//             WindowEvent::Focused(focused) => {
//                 if focused {
//                     println!("Hejsa");
//                 }
//             }
//             WindowEvent::KeyboardInput { .. } => {}
//             WindowEvent::ModifiersChanged(_) => {}
//             WindowEvent::CursorMoved { .. } => {}
//             WindowEvent::CursorEntered { .. } => {}
//             WindowEvent::CursorLeft { .. } => {}
//             WindowEvent::MouseWheel { .. } => {}
//             WindowEvent::MouseInput { .. } => {}
//             WindowEvent::TouchpadPressure { .. } => {}
//             WindowEvent::AxisMotion { .. } => {}
//             WindowEvent::Touch(_) => {}
//             WindowEvent::ScaleFactorChanged { .. } => {}
//             WindowEvent::ThemeChanged(_) => {}
//         },
//         Event::DeviceEvent { .. } => {}
//         Event::UserEvent(_) => {}
//         Event::Suspended => {}
//         Event::Resumed => {}
//         Event::MainEventsCleared => {}
//         Event::RedrawRequested(_) => {}
//         Event::RedrawEventsCleared => {}
//         Event::LoopDestroyed => {}
//     });
// }
fn main() {}