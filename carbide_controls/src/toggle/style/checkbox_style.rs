use crate::toggle::toggle_style::ToggleStyle;
use crate::toggle::toggle_value::ToggleValue;
use crate::toggle::ToggleAction;
use crate::UnfocusAction;
use carbide::color::TRANSPARENT;
use carbide::environment::{EnvironmentColor, IntoColorReadState};
use carbide::focus::Focus;
use carbide::state::{AnyReadState, AnyState, Map1, Map2, ReadState, State};
use carbide::widget::canvas::{Canvas, CanvasContext};
use carbide::widget::{AnyWidget, CornerRadii, HStack, IfElse, MouseArea, RoundedRectangle, Text, Widget, WidgetExt, ZStack};

#[derive(Debug, Clone)]
pub struct CheckboxStyle;

impl CheckboxStyle {
    fn create(focus: impl State<T=Focus>, value: impl State<T=ToggleValue>, enabled: impl ReadState<T=bool>, label: Box<dyn AnyReadState<T=String>>) -> impl Widget {
        MouseArea::new(Self::widget(focus.clone(), value.clone(), enabled.clone(), label))
            .custom_on_click(ToggleAction {
                value,
                focus: focus.clone(),
                enabled,
            }).custom_on_click_outside(UnfocusAction(focus.clone()))
            .focused(focus.clone())
    }

    fn widget(focus: impl State<T=Focus>, value: impl State<T=ToggleValue>, enabled: impl ReadState<T=bool>, label: Box<dyn AnyReadState<T=String>>) -> impl Widget {
        /*let background_color = Map1::read_map(value.clone(), |value| {
            match value {
                ToggleValue::True => EnvironmentColor::Green,
                ToggleValue::Mixed => EnvironmentColor::Blue,
                ToggleValue::False => EnvironmentColor::Red,
            }
        });

        let val = Map2::read_map(value.clone(), focus.clone(), |checked, focus| {
            format!("{:?}, {:?}", *checked, focus)
        });

        ZStack::new((
            Rectangle::new().fill(background_color),
            Text::new(val),
        ))*/

        let background_color = Map2::read_map(value.clone(), enabled.clone(), |value, enabled| {
            match *value {
                ToggleValue::True | ToggleValue::Mixed if *enabled => EnvironmentColor::Accent,
                ToggleValue::False if *enabled => EnvironmentColor::SecondarySystemBackground,
                ToggleValue::True | ToggleValue::Mixed | ToggleValue::False => EnvironmentColor::QuaternarySystemFill,
            }
        });

        let mark_color = Map1::read_map(enabled.clone(), |enabled| {
            if *enabled {
                EnvironmentColor::DarkText
            } else {
                EnvironmentColor::TertiaryLabel
            }
        });

        let mark_color2 = mark_color.clone();

        let label_color = Map1::read_map(enabled.clone(), |enabled| {
            if *enabled {
                EnvironmentColor::Label
            } else {
                EnvironmentColor::SecondaryLabel
            }
        });

        let checked_true =
            Map1::read_map(value.clone(), |value| *value == ToggleValue::True);

        let checked_intermediate =
            Map1::read_map(value.clone(), |value| *value == ToggleValue::Mixed);

        let border_color = Map2::read_map(
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

        let check_box = ZStack::new((
            RoundedRectangle::new(CornerRadii::all(3.0))
                .fill(background_color)
                .stroke(EnvironmentColor::OpaqueSeparator)
                .stroke_style(1.0)
                .boxed(),
            IfElse::new(checked_intermediate).when_true(Canvas::new(move |context: &mut CanvasContext| {
                context.move_to(4.0, 7.0);
                context.line_to(10.0, 7.0);
                context.set_stroke_style(mark_color.clone());
                context.set_line_width(2.0);
                context.stroke();
            })),
            IfElse::new(checked_true).when_true(Canvas::new(move |context: &mut CanvasContext| {
                context.move_to(4.0, 8.0);
                context.line_to(6.0, 10.0);
                context.line_to(10.0, 4.0);
                context.set_stroke_style(mark_color2.clone());
                context.set_line_width(2.0);
                context.stroke();
            })),
        ))
            .background(
                RoundedRectangle::new(CornerRadii::all(4.0))
                    .stroke(border_color)
                    .stroke_style(1.0)
                    .padding(-1.0)
            )
            .frame(14.0, 14.0);

        HStack::new((check_box, Text::new(label).color(label_color))).spacing(5.0)
    }
}

impl ToggleStyle for CheckboxStyle {
    fn create(&self, focus: Box<dyn AnyState<T=Focus>>, value: Box<dyn AnyState<T=ToggleValue>>, enabled: Box<dyn AnyReadState<T=bool>>, label: Box<dyn AnyReadState<T=String>>) -> Box<dyn AnyWidget> {
        CheckboxStyle::create(focus, value, enabled, label).boxed()
    }
}