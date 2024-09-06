use carbide_core::color::TRANSPARENT;
use carbide_core::draw::Rect;
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::environment::IntoColorReadState;
use carbide_core::focus::Focus;
use carbide_core::state::{IntoReadState, IntoState, LocalState, Map1, Map2, ReadState};
use carbide_core::widget::*;
use carbide_core::widget::canvas::{Canvas, CanvasContext};

use crate::{CheckBoxValue, EnabledState, PlainCheckBox, PlainCheckBoxDelegate};

pub struct CheckBox;

impl CheckBox {
    // TODO: Consider creating a newtype wrapper macro for CheckBox, wrapping plaincheckbox, to simplify the signature of the function
    pub fn new<L: IntoReadState<String>, C: IntoState<CheckBoxValue>>(label: L, checked: C) -> PlainCheckBox<LocalState<Focus>, C::Output, CheckBoxDelegate<L::Output>, EnabledState> {
        PlainCheckBox::new(checked)
            .delegate(CheckBoxDelegate { label: label.into_read_state() })
    }
}

#[derive(Clone)]
pub struct CheckBoxDelegate<L: ReadState<T=String>> {
    label: L,
}

impl<L: ReadState<T=String>> PlainCheckBoxDelegate for CheckBoxDelegate<L> {
    fn call(&self, focus: impl ReadState<T=Focus>, checked: impl ReadState<T=CheckBoxValue>, enabled: impl ReadState<T=bool>) -> Box<dyn AnyWidget> {
        let background_color = Map2::read_map(checked.clone(), enabled.clone(), |value, enabled| {
            match *value {
                CheckBoxValue::True | CheckBoxValue::Indeterminate if *enabled => EnvironmentColor::Accent,
                CheckBoxValue::False if *enabled => EnvironmentColor::SecondarySystemBackground,

                CheckBoxValue::True | CheckBoxValue::Indeterminate | CheckBoxValue::False => EnvironmentColor::QuaternarySystemFill,
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
        let label_color = mark_color.clone();

        let checked_true =
            Map1::read_map(checked.clone(), |value| *value == CheckBoxValue::True);

        let checked_intermediate =
            Map1::read_map(checked.clone(), |value| *value == CheckBoxValue::Indeterminate);

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
            IfElse::new(checked_intermediate).when_true(Canvas::new(move |_rect: Rect, context: &mut CanvasContext, _env: &mut Environment| {
                context.move_to(4.0, 7.0);
                context.line_to(10.0, 7.0);
                context.set_stroke_style(mark_color.clone());
                context.set_line_width(2.0);
                context.stroke();
            })),
            IfElse::new(checked_true).when_true(Canvas::new(move |_rect: Rect, context: &mut CanvasContext, _env: &mut Environment| {
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
            .frame(14.0, 14.0)
            .boxed();

        HStack::new((check_box, Text::new(self.label.clone()).color(label_color).boxed())).spacing(5.0).boxed()
    }
}