use std::hash::Hash;
use carbide_core::utils::clone_box;
use crate::identifiable::{AnyIdentifiableWidget, AnySelectableWidget, IdentifiableWidget};
use crate::picker::picker_selection::PickerSelection;
use crate::picker::style::PickerStyleKey;
use crate::picker::PickerStyle;
use crate::{enabled_state, EnabledState};
use carbide_core::draw::{Dimension, Position};
use carbide_core::focus::Focus;
use carbide_core::lifecycle::{InitializationContext, Initialize};
use carbide_core::state::{IntoReadState, LocalState, Map2, ReadState, ReadStateExtNew, State, StateContract, StateExtNew};
use carbide_core::widget::{AnyWidget, CommonWidget, Rectangle, Widget, WidgetExt, WidgetId, Sequence, ForEach};
use carbide_core::CommonWidgetImpl;
use crate::picker::picker_item::PickerItem;

#[derive(Clone, Widget, Debug)]
#[carbide_exclude(Initialize)]
pub struct Picker<T, F, M, E, L>
where
    T: StateContract + PartialEq + Eq + Hash,
    F: State<T=Focus>,
    E: ReadState<T=bool>,
    L: ReadState<T=String>,
    M: Sequence<dyn AnyIdentifiableWidget<T>>
{
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    child: Box<dyn AnyWidget>,
    model: M,

    #[state] focus: F,
    #[state] enabled: E,
    #[state] selected: PickerSelection<T>,
    #[state] label: L,
}

impl<
    T: StateContract + PartialEq + Eq + Hash,
    M: Sequence<dyn AnyIdentifiableWidget<T>>,
> Picker<T, LocalState<Focus>, M, EnabledState, String> {
    pub fn new<L: IntoReadState<String>>(label: L, selection: impl Into<PickerSelection<T>>, model: M) -> Picker<T, LocalState<Focus>, M, EnabledState, L::Output> {
        let focus = LocalState::new(Focus::Unfocused);

        Picker {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
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
    T: StateContract + PartialEq + Eq + Hash,
    F: State<T=Focus>,
    M: Sequence<dyn AnyIdentifiableWidget<T>>,
    E: ReadState<T=bool>,
    L: ReadState<T=String>,
> Picker<T, F, M, E, L> {
    fn selection(widget: &dyn AnyIdentifiableWidget<T>, selected: PickerSelection<T>) -> impl State<T=bool> {
        match selected.clone() {
            PickerSelection::Single(single) => {
                Map2::map(
                    clone_box(widget.identifier()).ignore_writes(),
                    single,
                    |value, selection| {
                        value == selection
                    },
                    |new, value, mut selection| {
                        if new {
                            *selection = value.clone();
                        }
                    }
                ).as_dyn()
            }
            PickerSelection::Optional(optional) => {
                Map2::map(
                    clone_box(widget.identifier()).ignore_writes(),
                    optional,
                    |value, selection| {
                        selection.as_ref().is_some_and(|x| x == value)
                    },
                    |new, value, mut selection| {
                        if new {
                            *selection = Some(value.clone());
                        } else {
                            *selection = None;
                        }
                    }
                ).as_dyn()
            }
            PickerSelection::Multi(multi) => {
                Map2::map(
                    clone_box(widget.identifier()).ignore_writes(),
                    multi,
                    |value, selection| {
                        selection.contains(value)
                    },
                    |new, value, mut selection| {
                        if new {
                            selection.insert(value.clone());
                        } else {
                            selection.remove(&*value);
                        }
                    }
                ).as_dyn()
            }
        }
    }
}

impl<
    T: StateContract + PartialEq + Eq + Hash,
    F: State<T=Focus>,
    M: Sequence<dyn AnyIdentifiableWidget<T>>,
    E: ReadState<T=bool>,
    L: ReadState<T=String>,
> Initialize for Picker<T, F, M, E, L> {
    fn initialize(&mut self, ctx: &mut InitializationContext) {
        if let Some(style) = ctx.env_stack.get::<PickerStyleKey>() {

            let selected_for_closure = self.selected.clone();

            let foreach = ForEach::custom_widget(
                self.model.clone(),
                move |widget: &dyn AnyIdentifiableWidget<T>| {
                    let selected = selected_for_closure.clone();
                    PickerItem {
                        selection: Self::selection(widget, selected),
                        inner: clone_box(widget),
                    }
                }
            );

            let selection_type = self.selected.to_type();
            self.child = style.create(self.focus.as_dyn(), self.enabled.as_dyn_read(), self.label.as_dyn_read(), Box::new(foreach), selection_type);
        }
    }
}

impl<
    T: StateContract + PartialEq + Eq + Hash,
    F: State<T=Focus>,
    M: Sequence<dyn AnyIdentifiableWidget<T>>,
    E: ReadState<T=bool>,
    L: ReadState<T=String>,
> CommonWidget for Picker<T, F, M, E, L> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension, focus: self.focus);
}
