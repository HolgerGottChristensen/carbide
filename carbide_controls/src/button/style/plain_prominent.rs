use carbide::color::{ColorExt, TRANSPARENT};
use carbide::draw::theme::Theme;
use carbide::environment::{EnvironmentColor, IntoColorReadState};
use carbide::focus::Focus;
use carbide::state::{AnyReadState, Map2, Map3, Map4, Map5};
use carbide::widget::{AnyWidget, CornerRadii, EdgeInsets, RoundedRectangle, WidgetExt};
use crate::button::style::ButtonStyle;

#[derive(Copy, Clone, Debug)]
pub struct PlainProminentStyle;

impl ButtonStyle for PlainProminentStyle {
    fn create(&self, label: Box<dyn AnyWidget>, focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        let label_color = Map4::read_map(EnvironmentColor::Accent.color(), EnvironmentColor::TertiaryLabel.color(), enabled, pressed, |color, disabled_color, enabled, pressed| {
            if !*enabled {
                *disabled_color
            } else if *pressed {
                color.lightened(0.1)
            } else {
                *color
            }
        });

        let outline_color = Map2::read_map(
            focus.clone(),
            EnvironmentColor::Accent.color(),
            |focus, color| {
                if *focus == Focus::Focused {
                    *color
                } else {
                    TRANSPARENT
                }
            }
        );

        label
            .foreground_color(label_color)
            .background(
                RoundedRectangle::new(CornerRadii::all(5.0))
                    .stroke(outline_color)
                    .stroke_style(1.0)
                    .padding(EdgeInsets::vertical_horizontal(-3.0, -5.0))
            )
            .boxed()
    }
}