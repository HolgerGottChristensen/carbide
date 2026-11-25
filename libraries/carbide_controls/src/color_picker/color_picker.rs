use std::any::TypeId;
use carbide::automatic_style::AutomaticStyle;
use crate::EnabledState;
use carbide::color::RED;
use carbide::CommonWidgetImpl;
use carbide::draw::{Color, Dimension, Position};
use carbide::environment::Environment;
use carbide::flags::WidgetFlag;
use carbide::focus::Focus;
use carbide::state::{AnyReadState, AnyState, IntoState, LocalState, ReadState, ReadStateExt, State, StateExtNew};
use carbide::widget::{AnyWidget, CommonWidget, Empty, IntoWidget, Rectangle, Widget, WidgetId, WidgetProperties, WidgetSync};
use carbide::widget::properties::WidgetKindSimple;
use crate::color_picker::ColorPickerStyleKey;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Sync)]
pub struct ColorPicker<V, L> where
    V: State<T=Color>,
    L: Widget + WidgetProperties<Kind=WidgetKindSimple>,
{
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    child: Box<dyn AnyWidget>,
    style_id: TypeId,
    state: V,
    label: L,

    focused: Box<dyn AnyState<T=Focus>>,
    enabled: Box<dyn AnyReadState<T=bool>>,
    hovered: Box<dyn AnyState<T=bool>>,
    pressed: Box<dyn AnyState<T=bool>>,
}

impl ColorPicker<Color, Empty> {
    pub fn new<L: IntoWidget, V: IntoState<Color>>(label: L, value: V) -> ColorPicker<V::Output, L::Output> where L::Output:  WidgetProperties<Kind=WidgetKindSimple> {
        ColorPicker {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child: Box::new(Rectangle::new().fill(RED)),
            style_id: TypeId::of::<()>(),
            state: value.into_state(),
            label: label.into_widget(),
            focused: LocalState::new(Focus::Unfocused).as_dyn(),
            enabled: EnabledState::new(true).as_dyn_read(),
            hovered: LocalState::new(false).as_dyn(),
            pressed: LocalState::new(false).as_dyn(),
        }
    }
}

impl<V: State<T=Color>, L: Widget + WidgetProperties<Kind=WidgetKindSimple>> WidgetSync for ColorPicker<V, L> {
    fn sync(&mut self, env: &mut Environment) {
        self.focused.sync(env);
        self.enabled.sync(env);
        self.hovered.sync(env);
        self.pressed.sync(env);

        let style = env.get::<ColorPickerStyleKey>().map(|a | &**a).unwrap_or(&AutomaticStyle);

        if style.key() != self.style_id {
            self.style_id = style.key();
            self.child = style.create(self.label.clone().boxed(), self.focused.clone().as_dyn(), self.enabled.clone().as_dyn_read(), self.hovered.clone().as_dyn_read(), self.pressed.clone().as_dyn_read(), self.state.clone().as_dyn());
        }
    }
}

impl<V: State<T=Color>, L: Widget + WidgetProperties<Kind=WidgetKindSimple>> CommonWidget for ColorPicker<V, L> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension, flag: WidgetFlag::FOCUSABLE, flexibility: 10, focus: self.focused);
}