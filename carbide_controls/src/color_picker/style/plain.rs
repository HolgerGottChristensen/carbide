use carbide::color::{Color, ColorExt, BLACK, TRANSPARENT};
use carbide::environment::{Environment, EnvironmentColor, IntoColorReadState};
use carbide::focus::{Focus, FocusManager, Refocus};
use carbide::state::{AnyReadState, AnyState, Map1, Map2, ReadState, State};
use carbide::text::text_wrap::Wrap;
use carbide::widget::{AnyWidget, CornerRadii, HStack, IfElse, MouseArea, MouseAreaAction, MouseAreaActionContext, Rectangle, RoundedRectangle, WidgetExt, ZStack};
use carbide_dialogs::color_dialog::ColorDialog;
use crate::color_picker::style::ColorPickerStyle;
use crate::UnfocusAction;

#[derive(Copy, Clone, Debug)]
pub struct PlainStyle;

impl ColorPickerStyle for PlainStyle {
    fn create(&self, label: Box<dyn AnyWidget>, focus: Box<dyn AnyState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>, value: Box<dyn AnyState<T=Color>>) -> Box<dyn AnyWidget> {

        let outline_color = Map2::read_map(EnvironmentColor::Accent.color(), focus.clone(), |color, focused| {
            if *focused == Focus::Focused {
                *color
            } else {
                TRANSPARENT
            }
        });

        let label_color = Map1::read_map(enabled.clone(), |enabled| {
            if *enabled {
                EnvironmentColor::DarkText
            } else {
                EnvironmentColor::TertiaryLabel
            }
        });

        let stack = HStack::new((
            label.foreground_color(label_color),
            ZStack::new((
                RoundedRectangle::new(CornerRadii::all(5.0))
                    .fill(value.clone()),
                IfElse::new(enabled.clone())
                    .when_false(RoundedRectangle::new(CornerRadii::all(5.0))
                        .stroke(BLACK.with_opacity(0.4))
                        .stroke_style(5.0))
            ))
                .background(
                    RoundedRectangle::new(CornerRadii::all(5.0))
                        .stroke(outline_color)
                        .stroke_style(1.0)
                        .padding(-2.0)
                )
                .frame(40.0, 22.0)
        )).spacing(5.0);

        MouseArea::new(stack)
            .custom_on_click(PlainColorPickerAction {
                value: value.clone(),
                focus: focus.clone(),
                enabled: enabled.clone(),
            })
            .custom_on_click_outside(UnfocusAction(focus.clone()))
            .focused(focus)
            .text_wrap(Wrap::None)
            .boxed()
    }
}

#[derive(Debug, Clone)]
pub struct PlainColorPickerAction<C, F, E> where
    C: State<T=Color>,
    F: State<T=Focus>,
    E: ReadState<T=bool>,
{
    pub value: C,
    pub focus: F,
    pub enabled: E,
}

impl<C: State<T=Color>, F: State<T=Focus>, E: ReadState<T=bool>> PlainColorPickerAction<C, F, E> {
    fn trigger(&mut self, env: &mut Environment) {
        self.enabled.sync(env);

        if !*self.enabled.value() {
            return;
        }

        self.focus.sync(env);
        self.value.sync(env);

        if *self.focus.value() != Focus::Focused {
            *self.focus.value_mut() = Focus::FocusRequested;
            FocusManager::get(env, |manager| {
                manager.request_focus(Refocus::FocusRequest)
            });
        }

        ColorDialog::new(self.value.clone(), true)
            .open(env);
    }
}

impl<C: State<T=Color>, F: State<T=Focus>, E: ReadState<T=bool>> MouseAreaAction for PlainColorPickerAction<C, F, E> {
    fn call(&mut self, ctx: MouseAreaActionContext) {
        self.trigger(ctx.env)
    }
}