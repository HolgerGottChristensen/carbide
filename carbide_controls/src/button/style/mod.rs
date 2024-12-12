mod plain;
mod plain_prominent;
mod bordered;
mod bordered_prominent;

use std::fmt::Debug;
use dyn_clone::{clone_trait_object, DynClone};
use carbide::environment::Key;
use carbide::focus::Focus;
use carbide::state::{AnyReadState, AnyState};
use carbide::widget::{AnyWidget, Widget};
pub use plain::PlainStyle;
pub use plain_prominent::PlainProminentStyle;
pub use bordered::BorderedStyle;
pub use bordered_prominent::BorderedProminentStyle;

#[derive(Debug, Copy, Clone)]
pub(crate) struct ButtonStyleKey;

impl Key for ButtonStyleKey {
    type Value = Box<dyn ButtonStyle>;
}

pub trait ButtonStyle: Debug + DynClone {
    fn create(
        &self,
        label: Box<dyn AnyWidget>,
        focus: Box<dyn AnyReadState<T=Focus>>,
        enabled: Box<dyn AnyReadState<T=bool>>,
        hovered: Box<dyn AnyReadState<T=bool>>,
        pressed: Box<dyn AnyReadState<T=bool>>,
    ) -> Box<dyn AnyWidget>;
}

clone_trait_object!(ButtonStyle);