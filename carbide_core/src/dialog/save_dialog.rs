use std::ffi::OsString;
use std::path::PathBuf;

use futures::future::Map;
use futures::FutureExt;
use oneshot::{Receiver, RecvError};

use crate::environment::Environment;
#[cfg(target_os = "macos")]
use crate::platform::mac::open_save_panel;
#[cfg(target_os = "windows")]
use crate::platform::windows::open_save_panel;

pub type FuturePath = Map<Receiver<Option<OsString>>, fn(Result<Option<OsString>, RecvError>) -> Option<PathBuf>>;

pub struct SaveDialog {}

impl SaveDialog {
    pub fn new() -> Self {
        SaveDialog {}
    }

    #[cfg(target_os = "macos")]
    pub fn open(self, env: &Environment) -> FuturePath {
        open_save_panel(env).map(|a| a.ok().flatten().map(|a| PathBuf::from(a)))
    }

    #[cfg(target_os = "windows")]
    pub fn open(mut self, env: &Environment) -> FuturePath {
        open_save_panel(env, self).map(|a| a.ok().flatten().map(|a| PathBuf::from(a)))
    }

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    pub fn open(mut self, env: &Environment) -> FuturePath {
        todo!()
    }
}