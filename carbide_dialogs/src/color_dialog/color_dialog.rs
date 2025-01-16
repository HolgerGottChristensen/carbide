use dyn_clone::clone_box;
use carbide::draw::{AutomaticStyle, Color};
use carbide::environment::EnvironmentStack;
use carbide::state::{AnyReadState, AnyState, IntoReadState, IntoState, ReadStateExtNew, StateExtNew, StateSync};
use carbide::widget::WidgetId;
use crate::color_dialog::style::ColorDialogStyleKey;

#[derive(Debug)]
pub struct ColorDialog {
    id: WidgetId,
    color: Box<dyn AnyState<T=Color>>,
    show_alpha: Box<dyn AnyReadState<T=bool>>
}

impl ColorDialog {
    pub fn new<C: IntoState<Color>, A: IntoReadState<bool>>(color: C, show_alpha: A) -> ColorDialog {
        ColorDialog {
            id: WidgetId::new(),
            color: color.into_state().as_dyn(),
            show_alpha: show_alpha.into_read_state().as_dyn_read(),
        }
    }

    pub fn open(mut self, env_stack: &mut EnvironmentStack) {
        self.color.sync(env_stack);
        self.show_alpha.sync(env_stack);

        let style = clone_box(env_stack.get::<ColorDialogStyleKey>().map(|a | &**a).unwrap_or(&AutomaticStyle));


        style.open(self.color, self.show_alpha, env_stack);
    }
}