use carbide_core::color::TRANSPARENT;
use carbide_core::environment::EnvironmentColor;
use carbide_core::focus::Focus;
use carbide_core::state::{AnyReadState, AnyState, IntoReadState, IntoState, LocalState, Map2, TState};
use carbide_core::widget::*;

use crate::PlainSlider;

pub struct Slider;

impl Slider {
    pub fn new<V: IntoState<f64>, S: IntoReadState<f64>, E: IntoReadState<f64>>(value: V, start: S, end: E) -> PlainSlider<TState<Focus>, V::Output, S::Output, E::Output, Option<f64>, Box<dyn Widget>, Box<dyn Widget>, Box<dyn Widget>> {
        let focus = LocalState::new(Focus::Unfocused);

        let mut plain = PlainSlider::new(value, start, end)
            .focused(focus)
            .background(Self::background)
            .track(Self::track)
            .thumb(Self::thumb);
        plain
    }

    fn background(state: Box<dyn AnyState<T=f64>>, start: Box<dyn AnyReadState<T=f64>>, end: Box<dyn AnyReadState<T=f64>>, steps: Box<dyn AnyReadState<T=Option<f64>>>, focus: Box<dyn AnyReadState<T=Focus>>) -> Box<dyn Widget> {
        let outline_color = Map2::read_map(focus, EnvironmentColor::Accent.color(), |focus, color| {
            if *focus == Focus::Focused {
                *color
            } else {
                TRANSPARENT
            }
        });

        Capsule::new()
            .fill(EnvironmentColor::SystemFill)
            .background(
                Capsule::new()
                    .stroke(outline_color)
                    .stroke_style(1.0)
                    .padding(-2.0)
            )
            .frame_fixed_height(5.0)
    }

    fn track(state: Box<dyn AnyState<T=f64>>, start: Box<dyn AnyReadState<T=f64>>, end: Box<dyn AnyReadState<T=f64>>, steps: Box<dyn AnyReadState<T=Option<f64>>>, focus: Box<dyn AnyReadState<T=Focus>>) -> Box<dyn Widget> {
        Capsule::new()
            .fill(EnvironmentColor::Accent)
            .frame_fixed_height(5.0)
    }

    fn thumb(state: Box<dyn AnyState<T=f64>>, start: Box<dyn AnyReadState<T=f64>>, end: Box<dyn AnyReadState<T=f64>>, steps: Box<dyn AnyReadState<T=Option<f64>>>, focus: Box<dyn AnyReadState<T=Focus>>) -> Box<dyn Widget> {
        /*let outline_color = Map2::read_map(focus, EnvironmentColor::Accent.color(), |focus, color| {
            if *focus == Focus::Focused {
                *color
            } else {
                TRANSPARENT
            }
        });*/

        Circle::new()
            .fill(EnvironmentColor::DarkText)
            /*.background(
                Circle::new()
                    .stroke(outline_color)
                    .stroke_style(1.0)
                    .padding(-2.0)
            )*/
            .frame(15.0, 15.0)
    }
}
