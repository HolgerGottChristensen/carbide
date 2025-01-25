use crate::file_type::FileType;
use crate::save_dialog::style::SaveDialogStyleKey;
use carbide::asynchronous::AsyncContext;
use carbide::draw::AutomaticStyle;
use carbide::environment::Environment;
use carbide::state::{IntoReadState, ReadState, ReadStateExtNew, StateSync};
use carbide_core::state::AnyReadState;
use carbide_core::widget::{Identifiable, WidgetId};
use dyn_clone::clone_box;
use oneshot::RecvError;
use std::path::PathBuf;

#[derive(Debug)]
pub struct SaveDialog {
    id: WidgetId,
    title: Box<dyn AnyReadState<T=Option<String>>>,
    message: Box<dyn AnyReadState<T=Option<String>>>,
    default_file_name: Box<dyn AnyReadState<T=Option<String>>>,
    prompt: Box<dyn AnyReadState<T=Option<String>>>,
    show_hidden_files: Box<dyn AnyReadState<T=bool>>,
    file_types: Box<dyn AnyReadState<T=Vec<FileType>>>,
    path: Box<dyn AnyReadState<T=Option<PathBuf>>>,
}

impl SaveDialog {
    pub fn new() -> SaveDialog {
        SaveDialog {
            id: WidgetId::new(),
            title: Box::new(None),
            message: Box::new(None),
            default_file_name: Box::new(None),
            prompt: Box::new(None),
            show_hidden_files: Box::new(false),
            file_types: Box::new(vec![]),
            path: Box::new(None),
        }
    }

    pub fn set_title<T: IntoReadState<Option<String>>>(mut self, title: T) -> SaveDialog {
        self.title = title.into_read_state().as_dyn_read();
        self
    }

    pub fn set_default_file_name<T: IntoReadState<Option<String>>>(mut self, default_file_name: T) -> SaveDialog {
        self.default_file_name = default_file_name.into_read_state().as_dyn_read();
        self
    }

    pub fn set_message<T: IntoReadState<Option<String>>>(mut self, message: T) -> SaveDialog {
        self.message = message.into_read_state().as_dyn_read();
        self
    }

    pub fn set_prompt<T: IntoReadState<Option<String>>>(mut self, prompt: T) -> SaveDialog {
        self.prompt = prompt.into_read_state().as_dyn_read();
        self
    }

    pub fn set_show_hidden_files<T: IntoReadState<bool>>(mut self, show_hidden_files: T) -> SaveDialog {
        self.show_hidden_files = show_hidden_files.into_read_state().as_dyn_read();
        self
    }

    pub fn set_path<T: IntoReadState<Option<PathBuf>>>(mut self, path: T) -> SaveDialog {
        self.path = path.into_read_state().as_dyn_read();
        self
    }

    /// Set the allowed file types that can be saved.
    /// The first extension will be the default, and if an extension other than the
    /// allowed are set, the default file extension is appended.
    pub fn set_file_types<T: IntoReadState<Vec<FileType>>>(mut self, file_types: T) -> SaveDialog {
        self.file_types = file_types.into_read_state().as_dyn_read();
        self
    }

    pub fn open(mut self, env: &mut Environment, f: impl Fn(Result<Option<PathBuf>, RecvError>, &mut AsyncContext) + 'static) {
        self.title.sync(env);
        self.message.sync(env);
        self.prompt.sync(env);
        self.default_file_name.sync(env);
        self.show_hidden_files.sync(env);
        self.path.sync(env);
        self.file_types.sync(env);

        let title = &*self.title.value();
        let message = &*self.message.value();
        let prompt = &*self.prompt.value();
        let default_file_name = &*self.default_file_name.value();
        let show_hidden_files = &*self.show_hidden_files.value();
        let path = &*self.path.value();
        let file_types = &*self.file_types.value();

        let style = clone_box(env.get::<SaveDialogStyleKey>().map(|a | &**a).unwrap_or(&AutomaticStyle));

        let callback = Box::new(f);

        style.open(
            title.clone(),
            message.clone(),
            prompt.clone(),
            default_file_name.clone(),
            show_hidden_files.clone(),
            path.clone(),
            file_types,
            callback,
            env
        );
    }
}

impl Identifiable for SaveDialog {
    fn id(&self) -> WidgetId {
        self.id
    }
}