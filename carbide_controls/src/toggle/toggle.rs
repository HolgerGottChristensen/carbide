use crate::toggle::toggle_value::ToggleValue;
use crate::{enabled_state, EnabledState, ToggleStyle};
use carbide::draw::{Dimension, Position};
use carbide::flags::WidgetFlag;
use carbide::focus::Focus;
use carbide::lifecycle::{InitializationContext, Initialize};
use carbide::state::{IntoReadState, IntoState, LocalState, ReadState, ReadStateExtNew, State, StateExtNew};
use carbide::widget::{AnyWidget, CommonWidget, Empty, Widget, WidgetExt, WidgetId};
use carbide::CommonWidgetImpl;
use crate::style::SwitchStyle;
use crate::toggle::ToggleStyleKey;

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(Initialize)]
pub struct Toggle<F, V, E, L> where
    F: State<T=Focus>,
    V: State<T=ToggleValue>,
    E: ReadState<T=bool>,
    L: ReadState<T=String>,
{
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    child: Box<dyn AnyWidget>,
    #[state] focus: F,
    #[state] enabled: E,
    #[state] value: V,
    #[state] label: L,
}

impl Toggle<LocalState<Focus>, ToggleValue, EnabledState, String> {
    pub fn new<L: IntoReadState<String>, C: IntoState<ToggleValue>>(label: L, value: C) -> Toggle<LocalState<Focus>, C::Output, EnabledState, L::Output> {
        let focus_state = LocalState::new(Focus::Unfocused);

        let enabled_state = enabled_state();

        Toggle {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child: Empty::new().boxed(),
            focus: focus_state,
            enabled: enabled_state,
            value: value.into_state(),
            label: label.into_read_state(),
        }
    }
}

impl<F: State<T=Focus>, V: State<T=ToggleValue>, E: ReadState<T=bool>, L: ReadState<T=String>> Initialize for Toggle<F, V, E, L> {
    fn initialize(&mut self, ctx: &mut InitializationContext) {
        if let Some(style) = ctx.env_stack.get::<ToggleStyleKey>() {
            self.child = style.create(self.focus.as_dyn(), self.value.as_dyn(), self.enabled.as_dyn_read(), self.label.as_dyn_read());
        } else {
            self.child = SwitchStyle.create(self.focus.as_dyn(), self.value.as_dyn(), self.enabled.as_dyn_read(), self.label.as_dyn_read());
        }
    }
}

impl<F: State<T=Focus>, V: State<T=ToggleValue>, E: ReadState<T=bool>, L: ReadState<T=String>> CommonWidget for Toggle<F, V, E, L> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flag: WidgetFlag::FOCUSABLE, flexibility: 10, focus: self.focus);
}