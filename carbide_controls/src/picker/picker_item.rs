use crate::identifiable::AnySelectableWidget;
use carbide::accessibility::Accessibility;
use carbide::render::Render;
use carbide::state::{AnyState, State};
use carbide::widget::{CommonWidget, Widget, WidgetId};
use carbide::ModifierWidgetImpl;
use std::hash::Hash;
use carbide::identifiable::Identifiable;

#[derive(Clone, Debug, Widget)]
pub struct PickerItem<W, S> where W: Widget, S: State<T=bool> {
    pub selection: S,
    pub inner: W,
}

impl<W: Widget, S: State<T=bool>> AnySelectableWidget for PickerItem<W, S> {
    fn selection(&self) -> &dyn AnyState<T=bool> {
        &self.selection
    }

    fn child(&mut self, index: usize) -> &mut dyn AnySelectableWidget {
        panic!("This widget can not have any selectable widget children")
    }

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {}

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {}
}

impl<W: Widget, S: State<T=bool>> Identifiable for PickerItem<W, S> {
    type Id = WidgetId;

    fn id(&self) -> WidgetId {
        self.inner.id()
    }
}

impl<W: Widget, S: State<T=bool>> CommonWidget for PickerItem<W, S> {
    ModifierWidgetImpl!(self, child: self.inner);
}