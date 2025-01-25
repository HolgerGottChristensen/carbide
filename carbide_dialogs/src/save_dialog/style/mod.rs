use crate::open_dialog::style::OpenDialogStyle;
use crate::save_dialog::style::macos::MacOSNativeSaveDialogStyle;
use crate::{FileType, NativeStyle};
use carbide::asynchronous::AsyncContext;
use carbide::draw::AutomaticStyle;
use carbide::environment::{Environment, EnvironmentKey};
use dyn_clone::{clone_trait_object, DynClone};
use oneshot::RecvError;
use std::fmt::Debug;
use std::path::PathBuf;

mod macos;

#[derive(Debug, Copy, Clone)]
pub(crate) struct SaveDialogStyleKey;

impl EnvironmentKey for SaveDialogStyleKey {
    type Value = Box<dyn SaveDialogStyle>;
}

pub trait SaveDialogStyle: Debug + DynClone + 'static {
    fn open(&self, title: Option<String>, message: Option<String>, prompt: Option<String>, default_file_name: Option<String>, show_hidden_files: bool, path: Option<PathBuf>, file_types: &[FileType], f: Box<dyn Fn(Result<Option<PathBuf>, RecvError>, &mut AsyncContext) + 'static>, env: &mut Environment);
}

clone_trait_object!(SaveDialogStyle);

impl SaveDialogStyle for AutomaticStyle {
    fn open(&self, title: Option<String>, message: Option<String>, prompt: Option<String>, default_file_name: Option<String>, show_hidden_files: bool, path: Option<PathBuf>, file_types: &[FileType], f: Box<dyn Fn(Result<Option<PathBuf>, RecvError>, &mut AsyncContext) + 'static>, env: &mut Environment) {
        SaveDialogStyle::open(&NativeStyle, title, message, prompt, default_file_name, show_hidden_files, path, file_types, f, env)
    }
}

impl SaveDialogStyle for NativeStyle {
    fn open(&self, title: Option<String>, message: Option<String>, prompt: Option<String>, default_file_name: Option<String>, show_hidden_files: bool, path: Option<PathBuf>, file_types: &[FileType], f: Box<dyn Fn(Result<Option<PathBuf>, RecvError>, &mut AsyncContext) + 'static>, env: &mut Environment) {
        MacOSNativeSaveDialogStyle.open(title, message, prompt, default_file_name, show_hidden_files, path, file_types, f, env)
    }
}

