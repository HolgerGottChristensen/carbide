use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::iter::Map;
use dyn_clone::{clone_trait_object, DynClone};
use carbide::automatic_style::AutomaticStyle;
use carbide::color::Color;
use carbide::environment::{EnvironmentColor, EnvironmentKey};
use carbide::focus::Focus;
use carbide::state::AnyReadState;
use carbide::widget::{AnyWidget, Capsule, Rectangle};
use crate::draw::DebugStyle;
use crate::state::Map2;
use crate::widget::{IfElse, Widget, WidgetExt};

#[derive(Debug, Copy, Clone)]
pub(crate) struct VerticalScrollBarStyleKey;

#[derive(Debug, Copy, Clone)]
pub(crate) struct HorizontalScrollBarStyleKey;

impl EnvironmentKey for VerticalScrollBarStyleKey {
    type Value = Box<dyn ScrollBarStyle>;
}

impl EnvironmentKey for HorizontalScrollBarStyleKey {
    type Value = Box<dyn ScrollBarStyle>;
}

pub trait ScrollBarStyle: Debug + DynClone + 'static {

    fn key(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn thumb(&self, dragging: Box<dyn AnyReadState<T=bool>>, hovering: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget>;

    fn background(&self, dragging: Box<dyn AnyReadState<T=bool>>, hovering: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget>;
}

impl ScrollBarStyle for AutomaticStyle {
    fn thumb(&self, dragging: Box<dyn AnyReadState<T=bool>>, hovering: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        let dragging_or_hovering = Map2::read_map(dragging, hovering, |dragging, hovering| {
            *hovering || *dragging
        });

        IfElse::new(dragging_or_hovering)
            .when_true(Capsule::new()
                .fill(EnvironmentColor::RegularLight)
                .stroke(EnvironmentColor::RegularDark)
                .stroke_style(1.0)
                .frame(8.0, 8.0))
            .when_false(Capsule::new()
                .fill(EnvironmentColor::ThinLight)
                .stroke(EnvironmentColor::ThinDark)
                .stroke_style(1.0)
                .frame(8.0, 8.0))
            .boxed()
    }

    fn background(&self, dragging: Box<dyn AnyReadState<T=bool>>, hovering: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        let hide = Map2::read_map(dragging, hovering, |dragging, hovering| {
            !hovering && !dragging
        });

        Rectangle::new()
            .fill(Color::Rgba(0.0, 0.0, 0.0, 0.5))
            .hidden(hide)
            .frame(8.0, 8.0)
            .boxed()
    }
}

impl ScrollBarStyle for DebugStyle {
    fn thumb(&self, dragging: Box<dyn AnyReadState<T=bool>>, hovering: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        Rectangle::new().fill(EnvironmentColor::Blue).frame(8.0, 8.0).boxed()
    }

    fn background(&self, dragging: Box<dyn AnyReadState<T=bool>>, hovering: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        Rectangle::new().fill(EnvironmentColor::Red).frame(8.0, 8.0).boxed()
    }
}

clone_trait_object!(ScrollBarStyle);