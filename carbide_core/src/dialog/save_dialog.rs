use std::ffi::{c_void, OsString};
use std::path::PathBuf;

use futures::future::{Map, Then};
use futures::FutureExt;
use oneshot::{Receiver, RecvError};

use crate::environment::Environment;
use crate::platform::mac::open_save_panel;

pub type FuturePath = Map<Receiver<Option<OsString>>, fn(Result<Option<OsString>, RecvError>) -> Option<PathBuf>>;

pub struct SaveDialog {}

impl SaveDialog {
    pub fn new() -> Self {
        SaveDialog {}
    }

    pub fn open(mut self, env: &Environment) -> FuturePath {
        open_save_panel(env).map(|a| a.ok().flatten().map(|a| PathBuf::from(a)))
    }
}