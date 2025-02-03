use carbide::color::TRANSPARENT;
use carbide::environment::{EnvironmentColor, IntoColorReadState};
use carbide::focus::Focus;
use carbide::state::{AnyReadState, AnyState, Map1, Map2};
use carbide::widget::{AnyWidget, Capsule, Circle, IfElse, RoundedRectangle, WidgetExt};
use crate::slider::style::SliderStyle;

#[derive(Copy, Clone, Debug)]
pub struct PlainStyle;

impl SliderStyle for PlainStyle {
    fn create_thumb(&self, focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, percent: Box<dyn AnyReadState<T=f64>>, stepped: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {

        let thumb_color = Map1::read_map(enabled, |enabled| {
            if *enabled {
                EnvironmentColor::DarkText
            } else {
                EnvironmentColor::TertiaryLabel
            }
        });

        IfElse::new(stepped)
            .when_true(RoundedRectangle::new(2.0).fill(thumb_color.clone()).frame(8.0, 15.0))
            .when_false(Circle::new().fill(thumb_color).frame(15.0, 15.0))
            .boxed()
    }

    fn create_track(&self, focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, percent: Box<dyn AnyReadState<T=f64>>, stepped: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
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

    fn create_background(&self, focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, percent: Box<dyn AnyReadState<T=f64>>, stepped: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
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
}