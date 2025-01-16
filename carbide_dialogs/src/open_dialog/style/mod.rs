use crate::file_type::FileType;
use crate::open_dialog::open_dialog::OpenPanelSelectionType;
use crate::open_dialog::style::macos::MacOSNativeOpenDialogStyle;
use crate::save_dialog::style::SaveDialogStyle;
use crate::NativeStyle;
use carbide::asynchronous::AsyncContext;
use carbide::draw::AutomaticStyle;
use carbide::environment::EnvironmentStack;
use carbide_core::environment::Key;
use dyn_clone::{clone_trait_object, DynClone};
use oneshot::RecvError;
use std::fmt::Debug;
use std::path::PathBuf;

mod macos;

#[derive(Debug, Copy, Clone)]
pub(crate) struct OpenDialogStyleKey;

impl Key for OpenDialogStyleKey {
    type Value = Box<dyn OpenDialogStyle>;
}

pub trait OpenDialogStyle: Debug + DynClone + 'static {
    fn open(&self, title: Option<String>, message: Option<String>, prompt: Option<String>, multiple_selection: bool, show_hidden_files: bool, selection_type: OpenPanelSelectionType, path: Option<PathBuf>, file_types: &[FileType], f: Box<dyn Fn(Result<Option<Vec<PathBuf>>, RecvError>, &mut AsyncContext) + 'static>, env_stack: &mut EnvironmentStack);
}

impl OpenDialogStyle for AutomaticStyle {
    fn open(&self, title: Option<String>, message: Option<String>, prompt: Option<String>, multiple_selection: bool, show_hidden_files: bool, selection_type: OpenPanelSelectionType, path: Option<PathBuf>, file_types: &[FileType], f: Box<dyn Fn(Result<Option<Vec<PathBuf>>, RecvError>, &mut AsyncContext) + 'static>, env_stack: &mut EnvironmentStack) {
        OpenDialogStyle::open(&NativeStyle, title, message, prompt, multiple_selection, show_hidden_files, selection_type, path, file_types, f, env_stack)
    }
}

impl OpenDialogStyle for NativeStyle {
    fn open(&self, title: Option<String>, message: Option<String>, prompt: Option<String>, multiple_selection: bool, show_hidden_files: bool, selection_type: OpenPanelSelectionType, path: Option<PathBuf>, file_types: &[FileType], f: Box<dyn Fn(Result<Option<Vec<PathBuf>>, RecvError>, &mut AsyncContext) + 'static>, env_stack: &mut EnvironmentStack) {
        MacOSNativeOpenDialogStyle.open(title, message, prompt, multiple_selection, show_hidden_files, selection_type, path, file_types, f, env_stack)
    }
}

clone_trait_object!(OpenDialogStyle);