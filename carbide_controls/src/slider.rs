// use carbide_core::environment::EnvironmentColor;
// use carbide_core::state::TState;
// use carbide_core::widget::*;
//
// use crate::PlainSlider;
//
// pub struct Slider;
//
// impl Slider {
//     pub fn new(value: impl Into<TState<f64>>, start: f64, end: f64) -> Box<PlainSlider> {
//         let mut plain = PlainSlider::new(value, start, end)
//             .background(Self::background)
//             .indicator(Self::indicator)
//             .thumb(Self::thumb);
//         plain
//     }
//
//     fn background() -> Box<dyn Widget> {
//         Capsule::new()
//             .fill(EnvironmentColor::SystemFill)
//             .frame_fixed_height(5.0)
//     }
//
//     fn indicator() -> Box<dyn Widget> {
//         Capsule::new()
//             .fill(EnvironmentColor::Accent)
//             .frame_fixed_height(5.0)
//     }
//
//     fn thumb() -> Box<dyn Widget> {
//         Circle::new()
//             .fill(EnvironmentColor::DarkText)
//             .frame(15.0, 15.0)
//     }
// }
