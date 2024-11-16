use std::marker::PhantomData;
use crate::picker::picker_selection::PickerSelection;
use carbide::draw::{Dimension, Position};
use carbide::flags::WidgetFlag;
use carbide::focus::Focus;
use carbide::state::{IntoReadState, LocalState, ReadState, ReadStateExtNew, State, StateContract, StateExtNew};
use carbide::widget::{AnyWidget, CommonWidget, Empty, Rectangle, Widget, WidgetExt, WidgetId, WidgetSequence};
use carbide::CommonWidgetImpl;
use carbide::lifecycle::{InitializationContext, Initialize};
use crate::{enabled_state, EnabledState};
use crate::identifiable::IdentifiableWidgetSequence;
use crate::picker::PickerStyle;
use crate::picker::style::{PickerStyleKey, TestSelectableWidgetSequence};
use crate::toggle::ToggleStyleKey;

#[derive(Clone, Widget, Debug)]
//#[carbide_exclude(Layout, MouseEvent, KeyboardEvent, Update)]
#[carbide_exclude(Initialize)]
pub struct Picker<T, F, M, E, L>
where
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    E: ReadState<T=bool>,
    L: ReadState<T=String>,
    M: IdentifiableWidgetSequence<T>
{
    id: WidgetId,
    position: LocalState<Position>,
    dimension: LocalState<Dimension>,

    child: Box<dyn AnyWidget>,
    model: M,

    #[state] focus: F,
    #[state] enabled: E,
    #[state] selected: PickerSelection<T>,
    #[state] label: L,
}

impl<
    T: StateContract + PartialEq,
    M: IdentifiableWidgetSequence<T>,
> Picker<T, LocalState<Focus>, M, EnabledState, String> {
    pub fn new<L: IntoReadState<String>>(label: L, selection: impl Into<PickerSelection<T>>, model: M) -> Picker<T, LocalState<Focus>, M, EnabledState, L::Output> {
        let focus = LocalState::new(Focus::Unfocused);

        Picker {
            id: WidgetId::new(),
            position: LocalState::new(Position::new(0.0, 0.0)),
            dimension: LocalState::new(Dimension::new(0.0, 0.0)),
            child: Rectangle::new().boxed(),
            model,
            focus,
            enabled: enabled_state(),
            selected: selection.into(),
            label: label.into_read_state(),
        }
    }
}

impl<
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    M: IdentifiableWidgetSequence<T>,
    E: ReadState<T=bool>,
    L: ReadState<T=String>,
> Initialize for Picker<T, F, M, E, L> {
    fn initialize(&mut self, ctx: &mut InitializationContext) {
        if let Some(style) = ctx.env_stack.get::<PickerStyleKey>() {
            let selected = TestSelectableWidgetSequence {
                selected: self.selected.clone(),
                inner: self.model.clone(),
            };
            style.as_ref().test(32);
            self.child = style.create(self.focus.as_dyn(), self.enabled.as_dyn_read(), self.label.as_dyn_read(), Box::new(selected));
        }
    }
}

fn h(t: &dyn PickerStyle) -> u32 { t.test(32) }

impl<
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    M: IdentifiableWidgetSequence<T>,
    E: ReadState<T=bool>,
    L: ReadState<T=String>,
> CommonWidget for Picker<T, F, M, E, L> {
    fn position(&self) -> Position {
        *self.position.value()
    }

    fn set_position(&mut self, position: Position) {
        self.position.set_value(position);
    }

    fn dimension(&self) -> Dimension {
        *self.dimension.value()
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension.set_value(dimension);
    }

    CommonWidgetImpl!(self, id: self.id, child: self.child, flag: WidgetFlag::FOCUSABLE, flexibility: 1, focus: self.focus);
}
