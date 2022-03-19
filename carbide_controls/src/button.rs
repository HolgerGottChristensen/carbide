use std::fmt::{Debug, Formatter};

use carbide_core::{Color, DeserializeOwned, Serialize};
use carbide_core::cursor::MouseCursor;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::flags::Flags;
use carbide_core::focus::Focus;
use carbide_core::prelude::{EnvironmentColor, Uuid};
use carbide_core::state::{BoolState, FocusState, LocalState, Map3, ReadState, State, StateExt, StringState, TState};
use carbide_core::widget::*;

use crate::{Action, PlainButton};

#[derive(Clone, Widget)]
pub struct Button {
    id: Id,
    #[state]
    focus: FocusState,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    is_primary: bool,
    click: Box<dyn Action>,
    #[state]
    is_hovered: BoolState,
    #[state]
    is_pressed: BoolState,
    #[state]
    label: StringState,
    hover_cursor: MouseCursor,
    pressed_cursor: Option<MouseCursor>,
}

impl Button {
    pub fn new<S: Into<StringState>>(text: S) -> Box<Self> {
        let label = text.into();
        let focus_state: FocusState = LocalState::new(Focus::Unfocused).into();
        let hover_state: BoolState = LocalState::new(false).into();
        let pressed_state: BoolState = LocalState::new(false).into();

        Self::new_internal(
            true,
            focus_state,
            hover_state,
            pressed_state,
            Box::new(|_, _| {}),
            label,
            MouseCursor::Hand,
            None,
        )
    }

    pub fn on_click(
        mut self,
        fire: impl Action + 'static,
    ) -> Box<Self> {
        self.click = Box::new(fire);
        Self::new_internal(
            self.is_primary,
            self.focus,
            self.is_hovered,
            self.is_pressed,
            self.click,
            self.label,
            self.hover_cursor,
            self.pressed_cursor,
        )
    }

    pub fn hover<K: Into<BoolState>>(mut self, is_hovered: K) -> Box<Self> {
        self.is_hovered = is_hovered.into();
        Self::new_internal(
            self.is_primary,
            self.focus,
            self.is_hovered,
            self.is_pressed,
            self.click,
            self.label,
            self.hover_cursor,
            self.pressed_cursor,
        )
    }

    pub fn pressed<K: Into<BoolState>>(mut self, pressed: K) -> Box<Self> {
        self.is_pressed = pressed.into();
        Self::new_internal(
            self.is_primary,
            self.focus,
            self.is_hovered,
            self.is_pressed,
            self.click,
            self.label,
            self.hover_cursor,
            self.pressed_cursor,
        )
    }

    pub fn hover_cursor(mut self, cursor: MouseCursor) -> Box<Self> {
        self.hover_cursor = cursor;
        Self::new_internal(
            self.is_primary,
            self.focus,
            self.is_hovered,
            self.is_pressed,
            self.click,
            self.label,
            self.hover_cursor,
            self.pressed_cursor,
        )
    }

    pub fn pressed_cursor(mut self, cursor: MouseCursor) -> Box<Self> {
        self.pressed_cursor = Some(cursor);
        Self::new_internal(
            self.is_primary,
            self.focus,
            self.is_hovered,
            self.is_pressed,
            self.click,
            self.label,
            self.hover_cursor,
            self.pressed_cursor,
        )
    }

    pub fn focused<K: Into<FocusState>>(mut self, focused: K) -> Box<Self> {
        self.focus = focused.into();
        Self::new_internal(
            self.is_primary,
            self.focus,
            self.is_hovered,
            self.is_pressed,
            self.click,
            self.label,
            self.hover_cursor,
            self.pressed_cursor,
        )
    }

    fn new_internal(
        is_primary: bool,
        focus_state: FocusState,
        hover_state: BoolState,
        pressed_state: BoolState,
        clicked: Box<dyn Action>,
        label: StringState,
        hover_cursor: MouseCursor,
        pressed_cursor: Option<MouseCursor>,
    ) -> Box<Self> {
        let normal_color = if is_primary {
            EnvironmentColor::Accent.state()
        } else {
            EnvironmentColor::SecondarySystemBackground.state()
        };

        let background_color = Map3::read_map(
            hover_state.clone(), pressed_state.clone(), normal_color,
            |hover: &bool, pressed: &bool, normal: &Color| {
                if *pressed {
                    return normal.darkened(0.05);
                }
                if *hover {
                    return normal.lightened(0.05);
                }

                *normal
            }).ignore_writes();

        let child = PlainButton::new(
            ZStack::new(vec![
                RoundedRectangle::new(CornerRadii::all(3.0))
                    .fill(background_color)
                    .stroke(EnvironmentColor::OpaqueSeparator)
                    .stroke_style(1.0),
                Text::new(label.clone()),
            ])
        )
            .hovered(hover_state.clone())
            .pressed(pressed_state.clone())
            .on_click(clicked.clone())
            .focused(focus_state.clone())
            .hover_cursor(hover_cursor);

        let child = if let Some(cursor) = pressed_cursor {
            child.pressed_cursor(cursor)
        } else {
            child
        };

        Box::new(Button {
            id: Uuid::new_v4(),
            focus: focus_state,
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            is_primary,
            click: clicked,
            is_hovered: hover_state,
            is_pressed: pressed_state,
            label,
            hover_cursor,
            pressed_cursor,
        })
    }
    /*
        fn new_internal(
            is_primary: bool,
            focus_state: FocusState<GS>,
            display_item: Box<dyn Widget<GS>>,
            local_state: TState<T, GS>,
            clicked: fn(
                myself: &mut PlainButton<T, GS>,
                env: &mut Environment<GS>,
                global_state: &mut GS,
            ),
        ) -> Box<Self> {
            let focus_color = TupleState3::new(
                focus_state.clone(),
                EnvironmentColor::OpaqueSeparator,
                EnvironmentColor::Accent,
            )
                .mapped(|(focus, primary_color, focus_color)| {
                    if focus == &Focus::Focused {
                        *focus_color
                    } else {
                        *primary_color
                    }
                });

            let hover_state = CommonState::new_local_with_key(&false);
            let pressed_state = CommonState::new_local_with_key(&false);

            let normal_color = if is_primary {
                EnvironmentColor::Accent
            } else {
                EnvironmentColor::SecondarySystemBackground
            };

            let background_color =
                TupleState3::new(hover_state.clone(), pressed_state.clone(), normal_color).mapped(
                    |(hover, pressed, normal_color)| {
                        if *pressed {
                            return normal_color.darkened(0.05);
                        }
                        if *hover {
                            return normal_color.lightened(0.05);
                        }

                        *normal_color
                    },
                );

            let child = PlainButton::new(ZStack::new(vec![
                RoundedRectangle::new(CornerRadii::all(3.0))
                    .fill(background_color)
                    .stroke(focus_color)
                    .stroke_style(1.0),
                display_item.clone(),
            ]))
                .local_state(local_state.clone())
                .focused(focus_state.clone())
                .on_click(clicked)
                .hover(hover_state)
                .pressed(pressed_state);

            Box::new(Button {
                id: Id::new_v4(),
                child,
                position: [0.0, 0.0],
                dimension: [235.0, 26.0],
                focus: focus_state,
                is_primary,
                local_state,
                on_click: clicked,
                display_item,
            })
        }*/
}

impl CommonWidget for Button {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn flag(&self) -> Flags {
        Flags::FOCUSABLE
    }

    fn get_focus(&self) -> Focus {
        self.focus.value().clone()
    }

    fn set_focus(&mut self, focus: Focus) {
        *self.focus.value_mut() = focus;
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

impl Debug for Button {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("child", &self.child)
            .finish()
    }
}

impl WidgetExt for Button {}
