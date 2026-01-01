use crate::file_type::FileType;
use crate::open_dialog::open_dialog::OpenPanelSelectionType;
#[cfg(target_os = "macos")]
use crate::open_dialog::style::macos::MacOSNativeOpenDialogStyle;
use crate::save_dialog::style::SaveDialogStyle;
use crate::NativeStyle;
use carbide::asynchronous::AsyncContext;
use carbide::draw::AutomaticStyle;
use carbide::environment::Environment;
use carbide_core::environment::EnvironmentKey;
use dyn_clone::{clone_trait_object, DynClone};
use oneshot::RecvError;
use std::fmt::Debug;
use std::path::PathBuf;

#[cfg(target_os = "macos")]
mod macos;

#[derive(Debug, Copy, Clone)]
pub(crate) struct OpenDialogStyleKey;

impl EnvironmentKey for OpenDialogStyleKey {
    type Value = Box<dyn OpenDialogStyle>;
}

pub trait OpenDialogStyle: Debug + DynClone + 'static {
    fn open(&self, title: Option<String>, message: Option<String>, prompt: Option<String>, multiple_selection: bool, show_hidden_files: bool, selection_type: OpenPanelSelectionType, path: Option<PathBuf>, file_types: &[FileType], f: Box<dyn Fn(Result<Option<Vec<PathBuf>>, RecvError>, &mut AsyncContext) + 'static>, env: &mut Environment);
}

impl OpenDialogStyle for AutomaticStyle {
    fn open(&self, title: Option<String>, message: Option<String>, prompt: Option<String>, multiple_selection: bool, show_hidden_files: bool, selection_type: OpenPanelSelectionType, path: Option<PathBuf>, file_types: &[FileType], f: Box<dyn Fn(Result<Option<Vec<PathBuf>>, RecvError>, &mut AsyncContext) + 'static>, env: &mut Environment) {
        OpenDialogStyle::open(&NativeStyle, title, message, prompt, multiple_selection, show_hidden_files, selection_type, path, file_types, f, env)
    }
}

#[cfg(target_os = "macos")]
impl OpenDialogStyle for NativeStyle {
    fn open(&self, title: Option<String>, message: Option<String>, prompt: Option<String>, multiple_selection: bool, show_hidden_files: bool, selection_type: OpenPanelSelectionType, path: Option<PathBuf>, file_types: &[FileType], f: Box<dyn Fn(Result<Option<Vec<PathBuf>>, RecvError>, &mut AsyncContext) + 'static>, env: &mut Environment) {
        MacOSNativeOpenDialogStyle.open(title, message, prompt, multiple_selection, show_hidden_files, selection_type, path, file_types, f, env)
    }
}

#[cfg(not(target_os = "macos"))]
impl OpenDialogStyle for NativeStyle {
    fn open(&self, title: Option<String>, message: Option<String>, prompt: Option<String>, multiple_selection: bool, show_hidden_files: bool, selection_type: OpenPanelSelectionType, path: Option<PathBuf>, file_types: &[FileType], f: Box<dyn Fn(Result<Option<Vec<PathBuf>>, RecvError>, &mut AsyncContext) + 'static>, env: &mut Environment) {
        todo!()
    }
}

clone_trait_object!(OpenDialogStyle);