use carbide::environment::IntoColorReadState;
use carbide::state::State;
use carbide_core::color::TRANSPARENT;
use carbide_core::environment::EnvironmentColor;
use carbide_core::focus::Focus;
use carbide_core::state::{AnyReadState, AnyState, IntoReadState, LocalState, Map1, Map2, ReadStateExtNew};
use carbide_core::widget::*;

use crate::{EnabledState, PlainSlider, SliderValue};

pub struct Slider;

impl Slider {
    pub fn new<V: SliderValue, St: State<T=V>, S: IntoReadState<V>, E: IntoReadState<V>>(value: St, start: S, end: E) -> PlainSlider<V, LocalState<Focus>, St, S::Output, E::Output, Option<V>, impl Widget, impl Widget, impl Widget, EnabledState> {
        let focus = LocalState::new(Focus::Unfocused);

        let plain = PlainSlider::new(value, start, end)
            .focused(focus)
            .background(Self::background)
            .track(Self::track)
            .thumb(Self::thumb);
        plain
    }

    fn background<V: SliderValue>(_state: Box<dyn AnyState<T=V>>, _start: Box<dyn AnyReadState<T=V>>, _end: Box<dyn AnyReadState<T=V>>, _steps: Box<dyn AnyReadState<T=Option<V>>>, focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>,) -> impl Widget {
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
    }

    fn track<V: SliderValue>(_state: Box<dyn AnyState<T=V>>, _start: Box<dyn AnyReadState<T=V>>, _end: Box<dyn AnyReadState<T=V>>, _steps: Box<dyn AnyReadState<T=Option<V>>>, _focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>,) -> impl Widget {
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
    }

    fn thumb<V: SliderValue>(_state: Box<dyn AnyState<T=V>>, _start: Box<dyn AnyReadState<T=V>>, _end: Box<dyn AnyReadState<T=V>>, steps: Box<dyn AnyReadState<T=Option<V>>>, _focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>,) -> impl Widget {
        let is_stepped = Map1::read_map(steps, |s| s.is_some());

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
    }
}
