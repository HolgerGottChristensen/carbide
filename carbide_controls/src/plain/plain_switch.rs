use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::flags::Flags;
use carbide_core::focus::Focus;
use carbide_core::focus::Refocus;
use carbide_core::state::{BoolState, FocusState, LocalState, MapOwnedState, State, StateKey, StringState};
use carbide_core::widget::{CommonWidget, HStack, Id, Rectangle, Spacer, Text, Widget, WidgetExt, WidgetIter, WidgetIterMut};

use crate::PlainButton;

#[derive(Debug, Clone, Widget)]
//#[focusable(block_focus)]
pub struct PlainSwitch {
    id: Id,
    #[state]
    focus: FocusState,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    delegate: fn(
        focus: FocusState,
        checked: BoolState,
        button: Box<dyn Widget>,
    ) -> Box<dyn Widget>,
    #[state]
    label: StringState,
    #[state]
    checked: BoolState,
}

impl PlainSwitch {
    pub fn focused<K: Into<FocusState>>(mut self, focused: K) -> Box<Self> {
        self.focus = focused.into();
        Self::new_internal(
            self.checked,
            self.focus,
            self.delegate,
            self.label,
        )
    }

    pub fn new<S: Into<StringState>, L: Into<BoolState>>(
        label: S,
        checked: L,
    ) -> Box<Self> {
        let focus_state = LocalState::new(Focus::Unfocused);

        let default_delegate = |_focus_state: FocusState,
                                checked: BoolState,
                                button: Box<dyn Widget>|
                                -> Box<dyn Widget> {
            let highlight_color = MapOwnedState::new(checked.clone(), |checked: &BoolState, env: &Environment| {
                if *checked.value() {
                    env.get_color(&StateKey::Color(EnvironmentColor::Green)).unwrap()
                } else {
                    env.get_color(&StateKey::Color(EnvironmentColor::Red)).unwrap()
                }
            });

            Rectangle::new(vec![button]).fill(highlight_color)
        };

        Self::new_internal(
            checked.into(),
            focus_state.into(),
            default_delegate,
            label.into(),
        )
    }

    pub fn delegate(
        self,
        delegate: fn(
            focus: FocusState,
            selected: BoolState,
            button: Box<dyn Widget>,
        ) -> Box<dyn Widget>,
    ) -> Box<Self> {
        let checked = self.checked;
        let focus_state = self.focus;
        let label_state = self.label;

        Self::new_internal(checked, focus_state, delegate, label_state)
    }

    fn new_internal(
        checked: BoolState,
        focus: FocusState,
        delegate: fn(
            focus: FocusState,
            selected: BoolState,
            button: Box<dyn Widget>,
        ) -> Box<dyn Widget>,
        label_state: StringState,
    ) -> Box<Self> {
        let button = PlainButton::new(Spacer::new())
            .on_click(capture!([checked, focus], |env: &mut Environment| {
                *checked = !*checked;

                if *focus != Focus::Focused {
                    *focus = Focus::FocusRequested;
                    env.request_focus(Refocus::FocusRequest);
                }
            }))
            .focused(focus.clone());

        let delegate_widget = delegate(focus.clone(), checked.clone(), button);

        let child = HStack::new(vec![
            delegate_widget,
            Text::new(label_state.clone()),
            Spacer::new(),
        ])
            .spacing(5.0);

        Box::new(PlainSwitch {
            id: Id::new_v4(),
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

impl CommonWidget for PlainSwitch {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
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
