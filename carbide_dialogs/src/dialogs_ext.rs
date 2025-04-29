use crate::open_dialog::style::{OpenDialogStyle, OpenDialogStyleKey};
use crate::save_dialog::style::{SaveDialogStyle, SaveDialogStyleKey};
use carbide::widget::{EnvUpdatingNew, Widget, WidgetExt};
use crate::color_dialog::style::{ColorDialogStyle, ColorDialogStyleKey};

pub trait DialogsExt: WidgetExt {
    fn color_dialog_style(self, value: impl ColorDialogStyle + 'static) -> impl Widget {
        EnvUpdatingNew::<Self, ColorDialogStyleKey>::new(Box::new(value) as Box<dyn ColorDialogStyle>, self)
    }

    fn open_dialog_style(self, value: impl OpenDialogStyle + 'static) -> impl Widget {
        EnvUpdatingNew::<Self, OpenDialogStyleKey>::new(Box::new(value) as Box<dyn OpenDialogStyle>, self)
    }

    fn save_dialog_style(self, value: impl SaveDialogStyle + 'static) -> impl Widget {
        EnvUpdatingNew::<Self, SaveDialogStyleKey>::new(Box::new(value) as Box<dyn SaveDialogStyle>, self)
    }
}

impl<T> DialogsExt for T where T: WidgetExt {}