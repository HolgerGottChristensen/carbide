use carbide::environment::IntoColorReadState;
use carbide_core::color::TRANSPARENT;
use carbide_core::environment::EnvironmentColor;
use carbide_core::focus::Focus;
use carbide_core::state::{AnyReadState, AnyState, IntoReadState, IntoState, LocalState, Map1, Map2, ReadStateExtNew};
use carbide_core::widget::*;

use crate::{EnabledState, PlainSlider};

pub struct Slider;

impl Slider {
    pub fn new<V: IntoState<f64>, S: IntoReadState<f64>, E: IntoReadState<f64>>(value: V, start: S, end: E) -> PlainSlider<LocalState<Focus>, V::Output, S::Output, E::Output, Option<f64>, Box<dyn AnyWidget>, Box<dyn AnyWidget>, Box<dyn AnyWidget>, EnabledState> {
        let focus = LocalState::new(Focus::Unfocused);

        let plain = PlainSlider::new(value, start, end)
            .focused(focus)
            .background(Self::background)
            .track(Self::track)
            .thumb(Self::thumb);
        plain
    }

    fn background(_state: Box<dyn AnyState<T=f64>>, _start: Box<dyn AnyReadState<T=f64>>, _end: Box<dyn AnyReadState<T=f64>>, _steps: Box<dyn AnyReadState<T=Option<f64>>>, focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>,) -> Box<dyn AnyWidget> {
        let outline_color = Map2::read_map(focus, EnvironmentColor::Accent.color(), |focus, color| {
            if *focus == Focus::Focused {
                *color
            } else {
                TRANSPARENT
            }
        });

        let background_color = Map1::read_map(enabled, |enabled| {
            if *enabled {
                EnvironmentColor::SystemFill
            } else {
                EnvironmentColor::TertiarySystemFill
            }
        });

        Capsule::new()
            .fill(background_color)
            .background(
                Capsule::new()
                    .stroke(outline_color)
                    .stroke_style(1.0)
                    .padding(-2.0)
            )
            .frame_fixed_height(5.0)
            .boxed()
    }

    fn track(_state: Box<dyn AnyState<T=f64>>, _start: Box<dyn AnyReadState<T=f64>>, _end: Box<dyn AnyReadState<T=f64>>, _steps: Box<dyn AnyReadState<T=Option<f64>>>, _focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>,) -> Box<dyn AnyWidget> {
        let track_color = Map1::read_map(enabled, |enabled| {
            if *enabled {
                EnvironmentColor::Accent
            } else {
                EnvironmentColor::SystemFill
            }
        });

        Capsule::new()
            .fill(track_color)
            .frame_fixed_height(5.0)
            .boxed()
    }

    fn thumb(_state: Box<dyn AnyState<T=f64>>, _start: Box<dyn AnyReadState<T=f64>>, _end: Box<dyn AnyReadState<T=f64>>, steps: Box<dyn AnyReadState<T=Option<f64>>>, _focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>,) -> Box<dyn AnyWidget> {
        let is_stepped = steps.map(|s| s.is_some());

        let thumb_color = Map1::read_map(enabled, |enabled| {
            if *enabled {
                EnvironmentColor::DarkText
            } else {
                EnvironmentColor::TertiaryLabel
            }
        });

        IfElse::new(is_stepped)
            .when_true(RoundedRectangle::new(2.0).fill(thumb_color.clone()).frame(8.0, 15.0))
            .when_false(Circle::new().fill(thumb_color).frame(15.0, 15.0))
            .boxed()
    }
}
