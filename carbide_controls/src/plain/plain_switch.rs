use carbide_core::Color;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Focusable};
use carbide_core::focus::Refocus;
use carbide_core::state::{BoolState, FocusState, LocalState, Map2, MapOwnedState, ReadState, State, StateKey, StringState, TState};
use carbide_core::widget::{CommonWidget, HStack, WidgetId, Rectangle, Spacer, Text, Widget, WidgetExt, WidgetIter, WidgetIterMut, ZStack};

use crate::PlainButton;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Focusable)]
pub struct PlainSwitch {
    id: WidgetId,
    #[state]
    focus: FocusState,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    delegate: fn(
        focus: FocusState,
        checked: BoolState,
    ) -> Box<dyn Widget>,
    #[state]
    label: StringState,
    #[state]
    checked: BoolState,
}

impl PlainSwitch {

    pub fn new<S: Into<StringState>, L: Into<BoolState>>(
        label: S,
        checked: L,
    ) -> Box<Self> {
        let focus_state = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            checked.into(),
            focus_state.into(),
            Self::default_delegate,
            label.into(),
        )
    }

    fn default_delegate(focus: TState<Focus>, checked: TState<bool>) -> Box<dyn Widget> {
        let background_color: TState<Color> = checked
            .choice(EnvironmentColor::Green.state(), EnvironmentColor::Red.state())
            .ignore_writes();

        let val = Map2::read_map(checked, focus, |checked: &bool, focus: &Focus| {
            format!("{:?}, {:?}", *checked, focus)
        }).ignore_writes();

        ZStack::new(vec![
            Rectangle::new().fill(background_color),
            Text::new(val),
        ])
    }

    pub fn delegate(
        self,
        delegate: fn(
            focus: FocusState,
            selected: BoolState,
        ) -> Box<dyn Widget>,
    ) -> Box<Self> {
        let checked = self.checked;
        let focus_state = self.focus;
        let label_state = self.label;

        Self::new_internal(checked, focus_state, delegate, label_state)
    }

    pub fn focused<K: Into<FocusState>>(mut self, focused: K) -> Box<Self> {
        self.focus = focused.into();
        Self::new_internal(
            self.checked,
            self.focus,
            self.delegate,
            self.label,
        )
    }

    fn new_internal(
        checked: BoolState,
        focus: FocusState,
        delegate: fn(
            focus: FocusState,
            selected: BoolState,
        ) -> Box<dyn Widget>,
        label_state: StringState,
    ) -> Box<Self> {
        let delegate_widget = delegate(focus.clone(), checked.clone());

        let button = PlainButton::new(delegate_widget)
            .on_click(capture!([checked, focus], |env: &mut Environment| {
                *checked = !*checked;

                if *focus != Focus::Focused {
                    *focus = Focus::FocusRequested;
                    env.request_focus(Refocus::FocusRequest);
                }
            }))
            .focused(focus.clone());

        let child = HStack::new(vec![
            button,
            Text::new(label_state.clone()),
        ])
            .spacing(5.0);

        Box::new(PlainSwitch {
            id: WidgetId::new_v4(),
            focus,
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            delegate,
            label: label_state,
            checked,
        })
    }
}

impl Focusable for PlainSwitch {
    fn focus_children(&self) -> bool {
        false
    }
}

impl CommonWidget for PlainSwitch {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }

    fn flag(&self) -> Flags {
        Flags::FOCUSABLE
    }

    fn children(&self) -> WidgetIter {
        if self.child.flag() == Flags::PROXY {
            self.child.children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        if self.child.flag() == Flags::PROXY {
            self.child.children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_focus(&self) -> Focus {
        self.focus.value().clone()
    }

    fn set_focus(&mut self, focus: Focus) {
        *self.focus.value_mut() = focus;
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl WidgetExt for PlainSwitch {}
