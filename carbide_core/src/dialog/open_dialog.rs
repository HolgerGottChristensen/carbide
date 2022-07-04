use std::ffi::OsString;
use std::path::PathBuf;

use futures::future::Map;
use futures::FutureExt;
use oneshot::{Receiver, RecvError};

use crate::dialog::FileSpecification;
use crate::environment::Environment;
#[cfg(target_os = "macos")]
use crate::platform::mac::open_open_panel;
#[cfg(target_os = "windows")]
use crate::platform::windows::open_open_panel;

pub type FuturePath = Map<
    Receiver<Option<Vec<OsString>>>,
    fn(Result<Option<Vec<OsString>>, RecvError>) -> Option<Vec<PathBuf>>,
>;

pub struct OpenDialog {
    allow_select_multiple: bool,
    allow_select_directories: bool,
    show_hidden: bool,
    title: Option<String>,
    button: Option<String>,
    message: Option<String>,
    starting_directory: Option<PathBuf>,
    packages_as_directories: bool,
    allowed_types: Option<Vec<FileSpecification>>,
    default_type: Option<FileSpecification>,
}

impl OpenDialog {
    pub fn new() -> Self {
        OpenDialog {
            allow_select_multiple: false,
            allow_select_directories: false,
            show_hidden: false,
            title: None,
            button: None,
            message: None,
            starting_directory: None,
            packages_as_directories: false,
            allowed_types: None,
            default_type: None,
        }
    }

    pub(crate) fn allow_select_multiple(&self) -> bool {
        self.allow_select_multiple
    }

    pub(crate) fn allow_select_directories(&self) -> bool {
        self.allow_select_directories
    }

    pub(crate) fn allow_show_hidden(&self) -> bool {
        self.show_hidden
    }

    pub(crate) fn allow_packages_as_directories(&self) -> bool {
        self.packages_as_directories
    }

    pub(crate) fn showing_title(&self) -> Option<&String> {
        self.title.as_ref()
    }

    pub(crate) fn showing_message(&self) -> Option<&String> {
        self.message.as_ref()
    }

    pub(crate) fn showing_default_button_text(&self) -> Option<&String> {
        self.button.as_ref()
    }

    pub(crate) fn showing_starting_directory(&self) -> Option<&PathBuf> {
        self.starting_directory.as_ref()
    }

    pub(crate) fn containing_default_type(&self) -> Option<&FileSpecification> {
        self.default_type.as_ref()
    }

    pub(crate) fn containing_allowed_types(&self) -> Option<&Vec<FileSpecification>> {
        self.allowed_types.as_ref()
    }

    /// Allow selection of more than a single element
    pub fn multi_selection(mut self) -> Self {
        self.allow_select_multiple = true;
        self
    }

    /// Allow selection of folders instead of files
    pub fn select_directories(mut self) -> Self {
        self.allow_select_directories = true;
        self
    }

    /// Show hidden files and folders
    pub fn show_hidden(mut self) -> Self {
        self.show_hidden = true;
        self
    }

    /// The title of the dialog. This is not shown for macos
    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    /// The name of the button that selects the files
    pub fn button_text(mut self, text: String) -> Self {
        self.button = Some(text);
        self
    }

    /// The message to show within the panel. This is well defined for macos, but not necessarily
    /// for other platforms.
    pub fn message(mut self, text: String) -> Self {
        self.message = Some(text);
        self
    }

    /// The starting directory of the panel
    pub fn starting_directory(mut self, path: PathBuf) -> Self {
        self.starting_directory = Some(path);
        self
    }

    /// The starting directory of the panel
    pub fn packages_as_directories(mut self) -> Self {
        self.packages_as_directories = true;
        self
    }

    /// The default type the open will choose.
    pub fn default_type(mut self, t: FileSpecification) -> Self {
        self.default_type = Some(t);
        self
    }

    /// The allowed types for the panel. If present the default type will also be inserted in the list.
    pub fn allowed_types(mut self, types: Vec<FileSpecification>) -> Self {
        self.allowed_types = Some(types);
        self
    }

    /*/// The label next to the input field. This is well defined for macos, but not necessarily
    /// for other platforms.
    pub fn name_field_label(mut self, text: String) -> Self {
        self.name_field_label = Some(text);
        self
    }*/

    #[cfg(target_os = "macos")]
    pub fn open(self, env: &Environment) -> FuturePath {
        open_open_panel(env, self).map(|a| {
            a.ok()
                .flatten()
                .map(|a| a.iter().map(|o| PathBuf::from(o)).collect::<Vec<_>>())
        })
    }

    #[cfg(target_os = "windows")]
    pub fn open(mut self, env: &Environment) -> FuturePath {
        open_open_panel(env, self).map(|a| {
            a.ok()
                .flatten()
                .map(|a| a.iter().map(|o| PathBuf::from(o)).collect::<Vec<_>>())
        })
    }

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    pub fn open(mut self, env: &Environment) -> FuturePath {
        todo!()
    }
}
