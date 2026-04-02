use std::any::TypeId;
use carbide::automatic_style::AutomaticStyle;
use crate::{EnabledState};
use carbide::color::RED;
use carbide::CommonWidgetImpl;
use carbide::draw::{Color, Dimension, Position};
use carbide::environment::Environment;
use carbide::flags::WidgetFlag;
use carbide::focus::Focus;
use carbide::lifecycle::{InitializationContext, Initialize};
use carbide::state::{IntoState, LocalState, ReadState, ReadStateExtNew, State, StateExtNew};
use carbide::widget::{Action, AnyWidget, CommonWidget, Empty, IntoWidget, MouseArea, Rectangle, Widget, WidgetId, WidgetStyle, WidgetSync};
use crate::button::{Button, ButtonStyleKey};
use crate::color_picker::ColorPickerStyleKey;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Sync)]
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
    style_id: TypeId,
    state: V,
    label: L,

    focused: F,
    enabled: E,
    hovered: H,
    pressed: P,
}

impl ColorPicker<LocalState<Focus>, Color, bool, LocalState<bool>, LocalState<bool>, Empty> {
    pub fn new<L: IntoWidget, V: IntoState<Color>>(label: L, value: V) -> ColorPicker<LocalState<Focus>, V::Output, impl ReadState<T=bool>, LocalState<bool>, LocalState<bool>, L::Output> {
        ColorPicker {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child: Box::new(Rectangle::new().fill(RED)),
            style_id: TypeId::of::<()>(),
            state: value.into_state(),
            label: label.into_widget(),
            focused: LocalState::new(Focus::Unfocused),
            enabled: EnabledState::new(true),
            hovered: LocalState::new(false),
            pressed: LocalState::new(false),
        }
    }
}

impl<F: State<T=Focus>, V: State<T=Color>, E: ReadState<T=bool>, H: State<T=bool>, P: State<T=bool>, L: Widget> WidgetSync for ColorPicker<F, V, E, H, P, L> {
    fn sync(&mut self, env: &mut Environment) {
        self.focused.sync(env);
        self.enabled.sync(env);
        self.hovered.sync(env);
        self.pressed.sync(env);

        let style = env.get::<ColorPickerStyleKey>().map(|a | &**a).unwrap_or(&AutomaticStyle);

        if style.key() != self.style_id {
            self.style_id = style.key();
            self.child = style.create(self.label.clone().boxed(), self.focused.as_dyn(), self.enabled.as_dyn_read(), self.hovered.as_dyn_read(), self.pressed.as_dyn_read(), self.state.as_dyn());
        }
    }
}

impl<F: State<T=Focus>, V: State<T=Color>, E: ReadState<T=bool>, H: State<T=bool>, P: State<T=bool>, L: Widget> CommonWidget for ColorPicker<F, V, E, H, P, L> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension, flag: WidgetFlag::FOCUSABLE, flexibility: 10, focus: self.focused);
}