use crate::identifiable::AnyIdentifiableWidget;
use crate::picker::picker_item::PickerItem;
use crate::picker::picker_selection::PickerSelection;
use crate::picker::style::PickerStyleKey;
use crate::picker::PickerStyle;
use crate::EnabledState;
use carbide::draw::AutomaticStyle;
use carbide::environment::Environment;
use carbide::widget::{WidgetStyle, WidgetSync};
use carbide_core::draw::{Dimension, Position};
use carbide_core::focus::Focus;
use carbide_core::state::{IntoReadState, LocalState, ReadState, ReadStateExtNew, State, StateContract, StateExtNew};
use carbide_core::widget::{AnyWidget, CommonWidget, ForEach, Rectangle, Sequence, Widget, WidgetExt, WidgetId};
use carbide_core::CommonWidgetImpl;
use std::any::TypeId;
use std::marker::PhantomData;

#[derive(Clone, Widget, Debug)]
#[carbide_exclude(Sync)]
pub struct Picker<T, F, M, E, L, S>
where
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    E: ReadState<T=bool>,
    L: ReadState<T=String>,
    M: Sequence<dyn AnyIdentifiableWidget<T=T>>,
    S: PickerSelection<T>
{
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    child: Box<dyn AnyWidget>,
    style_id: TypeId,

    model: M,

    focus: F,
    enabled: E,
    selected: S,
    label: L,
    phantom_data: PhantomData<T>,
}

impl<
    T: StateContract + PartialEq,
    M: Sequence<dyn AnyIdentifiableWidget<T=T>>,
> Picker<T, LocalState<Focus>, M, EnabledState, String, LocalState<T>> {
    pub fn new<L: IntoReadState<String>, S: PickerSelection<T>>(label: L, selection: S, model: M) -> Picker<T, LocalState<Focus>, M, EnabledState, L::Output, S> {
        let focus = LocalState::new(Focus::Unfocused);

        Picker {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child: Rectangle::new().boxed(),
            style_id: TypeId::of::<()>(),
            model,
            focus,
            enabled: EnabledState::new(true),
            selected: selection.into(),
            label: label.into_read_state(),
            phantom_data: Default::default(),
        }
    }
}

impl<
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    M: Sequence<dyn AnyIdentifiableWidget<T=T>>,
    E: ReadState<T=bool>,
    L: ReadState<T=String>,
    S: PickerSelection<T>
> WidgetSync for Picker<T, F, M, E, L, S> {
    fn sync(&mut self, env: &mut Environment) {
        self.focus.sync(env);
        self.enabled.sync(env);
        self.selected.sync(env);
        self.label.sync(env);

        let style = env.get::<PickerStyleKey>().map(|a | &**a).unwrap_or(&AutomaticStyle);

        if style.key() != self.style_id {
            self.style_id = style.key();

            let selected_for_closure = self.selected.clone();

            let foreach = ForEach::custom_widget(
                self.model.clone(),
                move |widget: &dyn AnyIdentifiableWidget<T=T>| {
                    let selected = selected_for_closure.clone();

                    let selected_state = selected.selection(widget);

                    PickerItem {
                        selection: selected_state,
                        inner: widget.as_widget().boxed(),
                    }
                }
            );

            let selection_type = self.selected.selection_type();
            self.child = style.create(self.focus.as_dyn(), self.enabled.as_dyn_read(), self.label.as_dyn_read(), Box::new(foreach), selection_type);
        }
    }
}

impl<
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    M: Sequence<dyn AnyIdentifiableWidget<T=T>>,
    E: ReadState<T=bool>,
    L: ReadState<T=String>,
    S: PickerSelection<T>
> CommonWidget for Picker<T, F, M, E, L, S> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension, focus: self.focus);
}
