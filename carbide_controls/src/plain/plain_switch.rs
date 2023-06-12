// use carbide_core::Color;
// use carbide_core::draw::{Dimension, Position};
// use carbide_core::environment::{Environment, EnvironmentColor};
// use carbide_core::flags::Flags;
// use carbide_core::focus::{Focus, Focusable};
// use carbide_core::focus::Refocus;
// use carbide_core::state::{LocalState, Map2, ReadState, State, TState};
// use carbide_core::widget::{
//     CommonWidget, HStack, Rectangle, Text, Widget, WidgetExt, WidgetId, WidgetIter,
//     WidgetIterMut, ZStack,
// };
//
// use crate::PlainButton;
//
// #[derive(Debug, Clone, Widget)]
// #[carbide_exclude(Focusable)]
// pub struct PlainSwitch {
//     id: WidgetId,
//     #[state]
//     focus: TState<Focus>,
//     child: Box<dyn Widget>,
//     position: Position,
//     dimension: Dimension,
//     delegate: fn(focus: TState<Focus>, checked: TState<bool>) -> Box<dyn Widget>,
//     #[state]
//     label: TState<String>,
//     #[state]
//     checked: TState<bool>,
// }
//
// impl PlainSwitch {
//     pub fn new(label: impl Into<TState<String>>, checked: impl Into<TState<bool>>) -> Box<Self> {
//         let focus_state = LocalState::new(Focus::Unfocused);
//
//         Self::new_internal(
//             checked.into(),
//             focus_state,
//             Self::default_delegate,
//             label.into(),
//         )
//     }
//
//     fn default_delegate(focus: TState<Focus>, checked: TState<bool>) -> Box<dyn Widget> {
//         let background_color: TState<Color> = checked
//             .choice(
//                 EnvironmentColor::Green.state(),
//                 EnvironmentColor::Red.state(),
//             )
//             .ignore_writes();
//
//         let val = Map2::read_map(checked, focus, |checked: &bool, focus: &Focus| {
//             format!("{:?}, {:?}", *checked, focus)
//         })
//         .ignore_writes();
//
//         ZStack::new(vec![
//             Rectangle::new().fill(background_color),
//             Text::new(val),
//         ])
//     }
//
//     pub fn delegate(
//         self,
//         delegate: fn(focus: TState<Focus>, selected: TState<bool>) -> Box<dyn Widget>,
//     ) -> Box<Self> {
//         let checked = self.checked;
//         let focus_state = self.focus;
//         let label_state = self.label;
//
//         Self::new_internal(checked, focus_state, delegate, label_state)
//     }
//
//     pub fn focused(mut self, focused: impl Into<TState<Focus>>) -> Box<Self> {
//         self.focus = focused.into();
//         Self::new_internal(self.checked, self.focus, self.delegate, self.label)
//     }
//
//     fn new_internal(
//         checked: TState<bool>,
//         focus: TState<Focus>,
//         delegate: fn(focus: TState<Focus>, selected: TState<bool>) -> Box<dyn Widget>,
//         label_state: TState<String>,
//     ) -> Box<Self> {
//         let delegate_widget = delegate(focus.clone(), checked.clone());
//
//         let button = PlainButton::new(delegate_widget)
//             .on_click(capture!([checked, focus], |env: &mut Environment| {
//                 *checked = !*checked;
//
//                 if *focus != Focus::Focused {
//                     *focus = Focus::FocusRequested;
//                     env.request_focus(Refocus::FocusRequest);
//                 }
//             }))
//             .focused(focus.clone());
//
//         let child = HStack::new(vec![button, Text::new(label_state.clone())]).spacing(5.0);
//
//         Box::new(PlainSwitch {
//             id: WidgetId::new(),
//             focus,
//             child,
//             position: Position::new(0.0, 0.0),
//             dimension: Dimension::new(0.0, 0.0),
//             delegate,
//             label: label_state,
//             checked,
//         })
//     }
// }
//
// impl Focusable for PlainSwitch {
//     fn focus_children(&self) -> bool {
//         false
//     }
// }
//
// impl CommonWidget for PlainSwitch {
//     fn id(&self) -> WidgetId {
//         self.id
//     }
//
//     fn flag(&self) -> Flags {
//         Flags::FOCUSABLE
//     }
//
//     fn children_mut(&mut self) -> WidgetIterMut {
//         if self.child.flag() == Flags::PROXY {
//             self.child.children_mut()
//         } else {
//             WidgetIterMut::single(&mut self.child)
//         }
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
//     fn children_direct(&mut self) -> WidgetIterMut {
//         WidgetIterMut::single(&mut self.child)
//     }
//
//     fn children_direct_rev(&mut self) -> WidgetIterMut {
//         WidgetIterMut::single(&mut self.child)
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
// impl WidgetExt for PlainSwitch {}
