use carbide::automatic_style::AutomaticStyle;
use crate::{EnabledState};
use carbide::color::RED;
use carbide::CommonWidgetImpl;
use carbide::draw::{Color, Dimension, Position};
use carbide::flags::WidgetFlag;
use carbide::focus::Focus;
use carbide::lifecycle::{InitializationContext, Initialize};
use carbide::state::{IntoState, LocalState, ReadState, ReadStateExtNew, State, StateExtNew};
use carbide::widget::{Action, AnyWidget, CommonWidget, Empty, IntoWidget, MouseArea, Rectangle, Widget, WidgetId};
use crate::button::{Button, ButtonStyleKey};
use crate::color_picker::ColorPickerStyleKey;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Initialize)]
pub struct ColorPicker<F, V, E, H, P, L> where
    F: State<T=Focus>,
    V: State<T=Color>,
    E: ReadState<T=bool>,
    H: State<T=bool>,
    P: State<T=bool>,
    L: Widget,
{
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    child: Box<dyn AnyWidget>,
    state: V,
    label: L,

    #[state] focus: F,
    #[state] enabled: E,
    #[state] hovered: H,
    #[state] pressed: P,
}

impl ColorPicker<LocalState<Focus>, Color, bool, LocalState<bool>, LocalState<bool>, Empty> {
    pub fn new<L: IntoWidget, V: IntoState<Color>>(label: L, value: V) -> ColorPicker<LocalState<Focus>, V::Output, impl ReadState<T=bool>, LocalState<bool>, LocalState<bool>, L::Output> {
        ColorPicker {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child: Box::new(Rectangle::new().fill(RED)),
            state: value.into_state(),
            label: label.into_widget(),
            focus: LocalState::new(Focus::Unfocused),
            enabled: EnabledState::new(true),
            hovered: LocalState::new(false),
            pressed: LocalState::new(false),
        }
    }
}

impl<F: State<T=Focus>, V: State<T=Color>, E: ReadState<T=bool>, H: State<T=bool>, P: State<T=bool>, L: Widget> Initialize for ColorPicker<F, V, E, H, P, L> {
    fn initialize(&mut self, ctx: &mut InitializationContext) {
        let style = ctx.env.get::<ColorPickerStyleKey>().map(|a | &**a).unwrap_or(&AutomaticStyle);

        let inner = style.create(self.label.clone().boxed(), self.focus.as_dyn(), self.enabled.as_dyn_read(), self.hovered.as_dyn_read(), self.pressed.as_dyn_read(), self.state.as_dyn());

        /*self.child = MouseArea::new(inner)
            .custom_on_click(crate::button::button::ButtonAction {
                action: self.action.clone(),
                focus: self.focus.clone(),
                enabled: self.enabled.clone(),
            })
            .custom_on_click_outside(crate::button::button::ButtonOutsideAction {
                action: self.action.clone(),
                focus: self.focus.clone(),
                enabled: self.enabled.clone(),
            })
            .focused(self.focus.clone())
            .pressed(self.pressed.clone())
            .hovered(self.hovered.clone())
            .hover_cursor(self.cursor)
            .boxed();*/

        self.child = inner;
    }
}

impl<F: State<T=Focus>, V: State<T=Color>, E: ReadState<T=bool>, H: State<T=bool>, P: State<T=bool>, L: Widget> CommonWidget for ColorPicker<F, V, E, H, P, L> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension, flag: WidgetFlag::FOCUSABLE, flexibility: 10, focus: self.focus);
}