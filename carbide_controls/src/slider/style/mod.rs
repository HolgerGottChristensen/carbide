mod plain;
mod unstyled;

use std::fmt::Debug;
use dyn_clone::{clone_trait_object, DynClone};
use carbide::draw::AutomaticStyle;
use carbide::environment::EnvironmentKey;
use carbide::focus::Focus;
use carbide::state::{AnyReadState, AnyState};
use carbide::widget::{AnySequence, AnyWidget};
pub use plain::PlainStyle;
pub use unstyled::UnstyledStyle;

#[derive(Debug, Copy, Clone)]
pub(crate) struct SliderStyleKey;

impl EnvironmentKey for SliderStyleKey {
    type Value = Box<dyn SliderStyle>;
}

pub trait SliderStyle: Debug + DynClone {
    fn create_thumb(
        &self,
        focus: Box<dyn AnyReadState<T=Focus>>,
        enabled: Box<dyn AnyReadState<T=bool>>,
        percent: Box<dyn AnyReadState<T=f64>>,
        stepped: Box<dyn AnyReadState<T=bool>>
    ) -> Box<dyn AnyWidget>;

    fn create_track(
        &self,
        focus: Box<dyn AnyReadState<T=Focus>>,
        enabled: Box<dyn AnyReadState<T=bool>>,
        percent: Box<dyn AnyReadState<T=f64>>,
        stepped: Box<dyn AnyReadState<T=bool>>
    ) -> Box<dyn AnyWidget>;

    fn create_background(
        &self,
        focus: Box<dyn AnyReadState<T=Focus>>,
        enabled: Box<dyn AnyReadState<T=bool>>,
        percent: Box<dyn AnyReadState<T=f64>>,
        stepped: Box<dyn AnyReadState<T=bool>>
    ) -> Box<dyn AnyWidget>;
}

impl SliderStyle for AutomaticStyle {
    fn create_thumb(&self, focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, percent: Box<dyn AnyReadState<T=f64>>, stepped: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        PlainStyle.create_thumb(focus, enabled, percent, stepped)
    }

    fn create_track(&self, focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, percent: Box<dyn AnyReadState<T=f64>>, stepped: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        PlainStyle.create_track(focus, enabled, percent, stepped)
    }

    fn create_background(&self, focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, percent: Box<dyn AnyReadState<T=f64>>, stepped: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        PlainStyle.create_background(focus, enabled, percent, stepped)
    }
}

clone_trait_object!(SliderStyle);