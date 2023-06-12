// use carbide_core::Color;
// use carbide_core::environment::EnvironmentColor;
// use carbide_core::focus::Focus;
// use carbide_core::state::{Map3, TState};
// use carbide_core::widget::*;
//
// use crate::PlainSwitch;
//
// pub struct Switch();
//
// impl Switch {
//     pub fn new(label: impl Into<TState<String>>, checked: impl Into<TState<bool>>) -> Box<PlainSwitch> {
//         let mut plain = PlainSwitch::new(label, checked.into()).delegate(Self::delegate);
//         /*
//         child = *child.delegate(|focus_state, checked_state, button: Box<dyn Widget<GS>>| {
//             let focus_color = TupleState3::new(
//                 focus_state,
//                 EnvironmentColor::OpaqueSeparator,
//                 EnvironmentColor::Accent,
//             )
//                 .mapped(|(focus, primary_color, focus_color)| {
//                     if focus == &Focus::Focused {
//                         *focus_color
//                     } else {
//                         *primary_color
//                     }
//                 });
//
//             let checked_color = TupleState3::new(
//                 checked_state.clone(),
//                 EnvironmentColor::SecondarySystemBackground,
//                 EnvironmentColor::Accent,
//             )
//                 .mapped(|(selected, primary_color, checked_color)| {
//                     if *selected {
//                         *checked_color
//                     } else {
//                         *primary_color
//                     }
//                 });
//
//
//         });
//
//         Box::new(Switch {
//             id: Id::new_v4(),
//             child,
//             position: [0.0, 0.0],
//             dimension: [235.0, 26.0],
//         })*/
//         plain
//     }
//
//     fn delegate(_focus: TState<Focus>, checked: TState<bool>) -> Box<dyn Widget> {
//         let checked_color = Map3::read_map(
//             checked.clone(),
//             EnvironmentColor::Accent.state(),
//             EnvironmentColor::SecondarySystemBackground.state(),
//             |check: &bool, checked_color: &Color, unchecked_color: &Color| {
//                 if *check {
//                     *checked_color
//                 } else {
//                     *unchecked_color
//                 }
//             },
//         )
//         .ignore_writes();
//
//         ZStack::new(vec![
//             Capsule::new()
//                 .fill(checked_color)
//                 .stroke(EnvironmentColor::OpaqueSeparator)
//                 .stroke_style(1.0),
//             IfElse::new(checked)
//                 .when_true(HStack::new(vec![
//                     Spacer::new(),
//                     Ellipse::new()
//                         .fill(EnvironmentColor::DarkText)
//                         .frame(20.0, 20.0),
//                 ]))
//                 .when_false(HStack::new(vec![
//                     Ellipse::new()
//                         .fill(EnvironmentColor::DarkText)
//                         .frame(18.0, 18.0),
//                     Spacer::new(),
//                 ]))
//                 .padding(2.0),
//         ])
//         .frame(39.0, 22.0)
//     }
// }
