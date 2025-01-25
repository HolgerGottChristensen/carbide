use crate::file_type::FileType;
use crate::open_dialog::style::OpenDialogStyle;
use crate::open_dialog::style::OpenDialogStyleKey;
use carbide::asynchronous::AsyncContext;
use carbide::draw::AutomaticStyle;
use carbide::environment::Environment;
use carbide::state::{IntoReadState, ReadState, ReadStateExtNew, StateSync};
use carbide_core::state::{AnyReadState, StateValue};
use carbide_core::widget::{Identifiable, WidgetId};
use dyn_clone::clone_box;
use oneshot::RecvError;
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, PartialEq, StateValue)]
pub enum OpenPanelSelectionType {
    File,
    Dictionary,
    FileAndDictionary
}

#[derive(Debug)]
pub struct OpenDialog {
    id: WidgetId,
    title: Box<dyn AnyReadState<T=Option<String>>>,
    message: Box<dyn AnyReadState<T=Option<String>>>,
    prompt: Box<dyn AnyReadState<T=Option<String>>>,
    multiple_selection: Box<dyn AnyReadState<T=bool>>,
    show_hidden_files: Box<dyn AnyReadState<T=bool>>,
    selection_type: Box<dyn AnyReadState<T=OpenPanelSelectionType>>,
    file_types: Box<dyn AnyReadState<T=Vec<FileType>>>,
    path: Box<dyn AnyReadState<T=Option<PathBuf>>>,
}

impl OpenDialog {
    pub fn new() -> OpenDialog {
        OpenDialog {
            id: WidgetId::new(),
            title: Box::new(None),
            message: Box::new(None),
            prompt: Box::new(None),
            multiple_selection: Box::new(true),
            show_hidden_files: Box::new(false),
            selection_type: Box::new(OpenPanelSelectionType::File),
            file_types: Box::new(vec![]),
            path: Box::new(None),
        }
    }

    pub fn set_title<T: IntoReadState<Option<String>>>(mut self, title: T) -> OpenDialog {
        self.title = title.into_read_state().as_dyn_read();
        self
    }

    pub fn set_message<T: IntoReadState<Option<String>>>(mut self, message: T) -> OpenDialog {
        self.message = message.into_read_state().as_dyn_read();
        self
    }

    pub fn set_prompt<T: IntoReadState<Option<String>>>(mut self, prompt: T) -> OpenDialog {
        self.prompt = prompt.into_read_state().as_dyn_read();
        self
    }

    pub fn set_multiple_selection<T: IntoReadState<bool>>(mut self, multiple_selection: T) -> OpenDialog {
        self.multiple_selection = multiple_selection.into_read_state().as_dyn_read();
        self
    }

    pub fn set_show_hidden_files<T: IntoReadState<bool>>(mut self, show_hidden_files: T) -> OpenDialog {
        self.show_hidden_files = show_hidden_files.into_read_state().as_dyn_read();
        self
    }

    pub fn set_selection_type<T: IntoReadState<OpenPanelSelectionType>>(mut self, selection_type: T) -> OpenDialog {
        self.selection_type = selection_type.into_read_state().as_dyn_read();
        self
    }

    pub fn set_path<T: IntoReadState<Option<PathBuf>>>(mut self, path: T) -> OpenDialog {
        self.path = path.into_read_state().as_dyn_read();
        self
    }

    pub fn set_file_types<T: IntoReadState<Vec<FileType>>>(mut self, file_types: T) -> OpenDialog {
        self.file_types = file_types.into_read_state().as_dyn_read();
        self
    }

    pub fn open(mut self, env: &mut Environment, f: impl Fn(Result<Option<Vec<PathBuf>>, RecvError>, &mut AsyncContext) + 'static) {
        self.title.sync(env);
        self.message.sync(env);
        self.prompt.sync(env);
        self.multiple_selection.sync(env);
        self.show_hidden_files.sync(env);
        self.selection_type.sync(env);
        self.path.sync(env);
        self.file_types.sync(env);

        let title = &*self.title.value();
        let message = &*self.message.value();
        let prompt = &*self.prompt.value();
        let multiple_selection = &*self.multiple_selection.value();
        let show_hidden_files = &*self.show_hidden_files.value();
        let selection_type = &*self.selection_type.value();
        let path = &*self.path.value();
        let file_types = &*self.file_types.value();

        let style = clone_box(env.get::<OpenDialogStyleKey>().map(|a | &**a).unwrap_or(&AutomaticStyle));

        let callback = Box::new(f);

        style.open(
            title.clone(),
            message.clone(),
            prompt.clone(),
            multiple_selection.clone(),
            show_hidden_files.clone(),
            selection_type.clone(),
            path.clone(),
            file_types,
            callback,
            env
        );
    }
}

impl Identifiable for OpenDialog {
    fn id(&self) -> WidgetId {
        self.id
    }
}