use crate::identifiable::AnySelectableWidget;
use carbide::accessibility::Accessibility;
use carbide::render::Render;
use carbide::state::{AnyReadState, AnyState, ReadState, State};
use carbide::widget::{AnyWidget, CommonWidget, Identifiable, Sequence, Widget, WidgetId};
use carbide::ModifierWidgetImpl;
use std::hash::Hash;

#[derive(Clone, Debug, Widget)]
pub struct PickerItem<W, S> where W: Widget, S: State<T=bool> {
    pub selection: S,
    pub inner: W,
}

impl<W: Widget, S: State<T=bool>> AnySelectableWidget for PickerItem<W, S> {
    fn selection(&self) -> &dyn AnyState<T=bool> {
        &self.selection
    }
}

impl<W: Widget, S: State<T=bool>> Identifiable for PickerItem<W, S> {
    fn id(&self) -> WidgetId {
        self.inner.id()
    }
}

impl<W: Widget, S: State<T=bool>> CommonWidget for PickerItem<W, S> {
    ModifierWidgetImpl!(self, child: self.inner);
}