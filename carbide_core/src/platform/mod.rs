//#[cfg(target_os = "macos")]
//pub mod mac;

#[cfg(target_os = "windows")]
pub(crate) mod windows;
