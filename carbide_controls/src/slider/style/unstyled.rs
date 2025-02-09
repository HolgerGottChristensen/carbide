use carbide::color::TRANSPARENT;
use carbide::environment::EnvironmentColor;
use carbide::focus::Focus;
use carbide::state::{AnyReadState, Map1, Map2, Map3};
use carbide::widget::{AnyWidget, Capsule, Circle, IfElse, Rectangle, RoundedRectangle, WidgetExt};
use crate::slider::style::SliderStyle;
use crate::SliderSteppingType;

#[derive(Copy, Clone, Debug)]
pub struct UnstyledStyle;

impl SliderStyle for UnstyledStyle {
    fn create_thumb(&self, focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, percent: Box<dyn AnyReadState<T=f64>>, stepped: Box<dyn AnyReadState<T=SliderSteppingType>>) -> Box<dyn AnyWidget> {
        let color = Map3::read_map(percent, stepped, enabled, |state, stepped, enabled| {
            if !*enabled {
                return EnvironmentColor::Teal;
            }

            if *state < 0.0 || *state > 1.0 {
                return EnvironmentColor::Purple;
            }

            if *stepped == SliderSteppingType::Stepped {
                return EnvironmentColor::Yellow;
            }

            if *stepped == SliderSteppingType::SmoothStepped {
                return EnvironmentColor::Pink;
            }

            EnvironmentColor::Blue
        });

        Rectangle::new().fill(color).frame(26.0, 26.0).boxed()
    }

    fn create_track(&self, focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, percent: Box<dyn AnyReadState<T=f64>>, stepped: Box<dyn AnyReadState<T=SliderSteppingType>>) -> Box<dyn AnyWidget> {
        Rectangle::new()
            .fill(EnvironmentColor::Green)
            .frame_fixed_height(26.0)
            .boxed()
    }

    fn create_background(&self, focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, percent: Box<dyn AnyReadState<T=f64>>, stepped: Box<dyn AnyReadState<T=SliderSteppingType>>) -> Box<dyn AnyWidget> {
        Rectangle::new()
            .fill(EnvironmentColor::Red)
            .frame_fixed_height(26.0)
            .boxed()
    }
}