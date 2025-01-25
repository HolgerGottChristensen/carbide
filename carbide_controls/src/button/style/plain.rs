use carbide::color::ColorExt;
use carbide::environment::{EnvironmentColor, IntoColorReadState};
use carbide::focus::Focus;
use carbide::state::{AnyReadState, Map5};
use carbide::widget::{AnyWidget, WidgetExt};
use crate::button::style::ButtonStyle;

#[derive(Copy, Clone, Debug)]
pub struct PlainStyle;

impl ButtonStyle for PlainStyle {
    fn create(&self, label: Box<dyn AnyWidget>, focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        let label_color = Map5::read_map(EnvironmentColor::Label.color(), EnvironmentColor::TertiaryLabel.color(), enabled, hovered, pressed, |color, disabled_color, enabled, hovered, pressed| {
            if !*enabled {
                *disabled_color
            } else if *pressed {
                color.darkened(0.1)
            } else if *hovered {
                color.lightened(0.1)
            } else {
                *color
            }
        });

        label
            .foreground_color(label_color)
            .boxed()
    }
}