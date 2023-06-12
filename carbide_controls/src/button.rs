// use std::fmt::{Debug, Formatter};
//
// use carbide_core::cursor::MouseCursor;
// use carbide_core::draw::{Dimension, Position};
// use carbide_core::flags::Flags;
// use carbide_core::focus::Focus;
// use carbide_core::environment::EnvironmentColor;
// use carbide_core::state::{LocalState, Map3, ReadState, State, TState};
// use carbide_core::widget::*;
// use carbide_core::Color;
//
// use crate::PlainButton;
// use carbide_core::widget::Action;
//
// #[derive(Clone, Widget)]
// pub struct Button {
//     id: WidgetId,
//     #[state]
//     focus: TState<Focus>,
//     child: Box<dyn Widget>,
//     position: Position,
//     dimension: Dimension,
//     is_primary: bool,
//     click: Box<dyn Action>,
//     #[state]
//     is_hovered: TState<bool>,
//     #[state]
//     is_pressed: TState<bool>,
//     #[state]
//     label: TState<String>,
//     hover_cursor: MouseCursor,
//     pressed_cursor: Option<MouseCursor>,
// }
//
// impl Button {
//     pub fn new(text: impl Into<TState<String>>) -> Box<Self> {
//         let label = text.into();
//         let focus_state = LocalState::new(Focus::Unfocused);
//         let hover_state = LocalState::new(false);
//         let pressed_state = LocalState::new(false);
//
//         Self::new_internal(
//             true,
//             focus_state,
//             hover_state,
//             pressed_state,
//             Box::new(|_, _| {}),
//             label,
//             MouseCursor::Hand,
//             None,
//         )
//     }
//
//     /// |env: &mut Environment, modifier_key: ModifierKey| {}
//     pub fn on_click(mut self, fire: impl Action + 'static) -> Box<Self> {
//         self.click = Box::new(fire);
//         Self::new_internal(
//             self.is_primary,
//             self.focus,
//             self.is_hovered,
//             self.is_pressed,
//             self.click,
//             self.label,
//             self.hover_cursor,
//             self.pressed_cursor,
//         )
//     }
//
//     pub fn hover(mut self, is_hovered: impl Into<TState<bool>>) -> Box<Self> {
//         self.is_hovered = is_hovered.into();
//         Self::new_internal(
//             self.is_primary,
//             self.focus,
//             self.is_hovered,
//             self.is_pressed,
//             self.click,
//             self.label,
//             self.hover_cursor,
//             self.pressed_cursor,
//         )
//     }
//
//     pub fn pressed(mut self, pressed: impl Into<TState<bool>>) -> Box<Self> {
//         self.is_pressed = pressed.into();
//         Self::new_internal(
//             self.is_primary,
//             self.focus,
//             self.is_hovered,
//             self.is_pressed,
//             self.click,
//             self.label,
//             self.hover_cursor,
//             self.pressed_cursor,
//         )
//     }
//
//     pub fn hover_cursor(mut self, cursor: MouseCursor) -> Box<Self> {
//         self.hover_cursor = cursor;
//         Self::new_internal(
//             self.is_primary,
//             self.focus,
//             self.is_hovered,
//             self.is_pressed,
//             self.click,
//             self.label,
//             self.hover_cursor,
//             self.pressed_cursor,
//         )
//     }
//
//     pub fn pressed_cursor(mut self, cursor: MouseCursor) -> Box<Self> {
//         self.pressed_cursor = Some(cursor);
//         Self::new_internal(
//             self.is_primary,
//             self.focus,
//             self.is_hovered,
//             self.is_pressed,
//             self.click,
//             self.label,
//             self.hover_cursor,
//             self.pressed_cursor,
//         )
//     }
//
//     pub fn focused(mut self, focused: impl Into<TState<Focus>>) -> Box<Self> {
//         self.focus = focused.into();
//         Self::new_internal(
//             self.is_primary,
//             self.focus,
//             self.is_hovered,
//             self.is_pressed,
//             self.click,
//             self.label,
//             self.hover_cursor,
//             self.pressed_cursor,
//         )
//     }
//
//     fn new_internal(
//         is_primary: bool,
//         focus_state: TState<Focus>,
//         hover_state: TState<bool>,
//         pressed_state: TState<bool>,
//         clicked: Box<dyn Action>,
//         label: TState<String>,
//         hover_cursor: MouseCursor,
//         pressed_cursor: Option<MouseCursor>,
//     ) -> Box<Self> {
//         let normal_color = if is_primary {
//             EnvironmentColor::Accent.state()
//         } else {
//             EnvironmentColor::SecondarySystemBackground.state()
//         };
//
//         let background_color = Map3::read_map(
//             hover_state.clone(),
//             pressed_state.clone(),
//             normal_color,
//             |hover: &bool, pressed: &bool, normal: &Color| {
//                 if *pressed {
//                     return normal.darkened(0.05);
//                 }
//                 if *hover {
//                     return normal.lightened(0.05);
//                 }
//
//                 *normal
//             },
//         )
//         .ignore_writes();
//
//         let child = PlainButton::new(ZStack::new(vec![
//             RoundedRectangle::new(CornerRadii::all(3.0))
//                 .fill(background_color)
//                 .stroke(EnvironmentColor::OpaqueSeparator)
//                 .stroke_style(1.0),
//             Text::new(label.clone()),
//         ]))
//         .hovered(hover_state.clone())
//         .pressed(pressed_state.clone())
//         .on_click(clicked.clone())
//         .focused(focus_state.clone())
//         .hover_cursor(hover_cursor);
//
//         let child = if let Some(cursor) = pressed_cursor {
//             child.pressed_cursor(cursor)
//         } else {
//             child
//         };
//
//         Box::new(Button {
//             id: WidgetId::new(),
//             focus: focus_state,
//             child,
//             position: Position::new(0.0, 0.0),
//             dimension: Dimension::new(100.0, 100.0),
//             is_primary,
//             click: clicked,
//             is_hovered: hover_state,
//             is_pressed: pressed_state,
//             label,
//             hover_cursor,
//             pressed_cursor,
//         })
//     }
// }
//
// impl CommonWidget for Button {
//     fn id(&self) -> WidgetId {
//         self.id
//     }
//
//     fn flag(&self) -> Flags {
//         Flags::FOCUSABLE
//     }
//
//     fn get_focus(&self) -> Focus {
//         self.focus.value().clone()
//     }
//
//     fn set_focus(&mut self, focus: Focus) {
//         *self.focus.value_mut() = focus;
//     }
//
//     fn position(&self) -> Position {
//         self.position
//     }
//
//     fn set_position(&mut self, position: Position) {
//         self.position = position;
//     }
//
//     fn dimension(&self) -> Dimension {
//         self.dimension
//     }
//
//     fn set_dimension(&mut self, dimension: Dimension) {
//         self.dimension = dimension
//     }
// }
//
// impl Debug for Button {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Button")
//             .field("child", &self.child)
//             .finish()
//     }
// }
//
// impl WidgetExt for Button {}
