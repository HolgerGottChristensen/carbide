mod plain;
mod inset;

use dyn_clone::{clone_trait_object, DynClone};
use carbide::draw::{AutomaticStyle, Scalar};
use carbide::environment::{EnvironmentKey, EnvironmentKeyDefault};
use carbide::widget::{AnySequence, AnyWidget, Sequence, WidgetStyle};
use carbide::widget::foreach_widget::Delegate;
use crate::identifiable::{AnyIdentifiableWidget, AnySelectableWidget};
pub use plain::*;
use crate::list::row_delegate::{RowDelegate, SelectableRowDelegate};
use crate::list::style::inset::InsetStyle;

#[derive(Debug, Copy, Clone)]
pub struct ListStyleKey;

impl EnvironmentKey for ListStyleKey {
    type Value = Box<dyn ListStyle>;
}

impl EnvironmentKeyDefault for ListStyleKey {
    fn default() -> Self::Value {
        Box::new(AutomaticStyle)
    }
}

pub trait ListStyle: WidgetStyle {
    fn base(&self, sequence: Box<dyn AnySequence>) -> Box<dyn AnyWidget>;

    fn requires_row_wrapping(&self) -> bool {
        true
    }

    fn row(&self) -> RowDelegate;

    fn selectable_row(&self) -> SelectableRowDelegate;
}

impl ListStyle for AutomaticStyle {
    fn base(&self, sequence: Box<dyn AnySequence>) -> Box<dyn AnyWidget> {
        ListStyle::base(&InsetStyle, sequence)
    }

    fn requires_row_wrapping(&self) -> bool {
        ListStyle::requires_row_wrapping(&InsetStyle)
    }

    fn row(&self) -> RowDelegate {
        ListStyle::row(&InsetStyle)
    }

    fn selectable_row(&self) -> SelectableRowDelegate {
        ListStyle::selectable_row(&InsetStyle)
    }
}

clone_trait_object!(ListStyle);