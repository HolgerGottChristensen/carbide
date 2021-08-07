// //! A simple, non-interactive **Polygon** widget for drawing arbitrary convex shapes.
//
// use crate::Point;
// use crate::draw::shape::triangle::Triangle;
//
//
// /// A basic, non-interactive, arbitrary **Polygon** widget.
// ///
// /// The **Polygon** is described by specifying its corners in order.
// ///
// /// **Polygon** will automatically close all shapes, so the given list of points does not need to
// /// start and end with the same position.
// #[derive(Copy, Clone, Debug)]
// pub struct Polygon<I> {
//     /// The points describing the corners of the **Polygon**.
//     pub points: I,
//     /// Whether or not the points should be automatically centred to the widget position.
//     pub maybe_shift_to_centre_from: Option<Point>,
// }
//
// /// Unique state for the **Polygon**.
// #[derive(Clone, Debug, PartialEq)]
// pub struct State {
//     /// Whether the rectangle is drawn as an outline or a filled color.
//     kind: Kind,
//     /// An owned version of the points yielded by the **Polygon**'s `points` iterator.
//     pub points: Vec<Point>,
// }
//
// /// Whether the rectangle is drawn as an outline or a filled color.
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub enum Kind {
//     /// Only the outline of the rectangle is drawn.
//     Outline,
//     /// The rectangle area is filled with some color.
//     Fill,
// }
//
// /// An iterator that triangulates a polygon represented by a sequence of points describing its
// /// edges.
// #[derive(Clone)]
// pub struct Triangles<I> {
//     first: Point,
//     prev: Point,
//     points: I,
// }
//
//
// impl<I> Polygon<I> {
//
// }
//
//
// /*impl<I> OldWidget for Polygon<I>
//     where I: IntoIterator<Item=Point>,
// {
//     type State = State;
//     type Style = Style;
//     type Event = ();
//
//     fn init_state(&self, _: widget::id::Generator) -> Self::State {
//         State {
//             kind: Kind::Fill,
//             points: Vec::new(),
//         }
//     }
//
//     fn style(&self) -> Self::Style {
//         self.style.clone()
//     }
//
//     fn is_over(&self) -> widget::IsOverFn {
//         is_over_widget
//     }
//
//     /// Update the state of the Polygon.
//     fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
//         use utils::{iter_diff, IterDiff};
//         let widget::UpdateArgs { rect, state, style, .. } = args;
//         let Polygon { points, maybe_shift_to_centre_from, .. } = self;
//
//         // A function that compares the given points iterator to the points currently owned by
//         // `State` and updates only if necessary.
//         fn update_points<I>(state: &mut widget::State<State>, points: I)
//             where I: IntoIterator<Item=Point>,
//         {
//             match iter_diff(&state.points, points) {
//                 Some(IterDiff::FirstMismatch(i, mismatch)) => state.update(|state| {
//                     state.points.truncate(i);
//                     state.points.extend(mismatch);
//                 }),
//                 Some(IterDiff::Longer(remaining)) =>
//                     state.update(|state| state.points.extend(remaining)),
//                 Some(IterDiff::Shorter(total)) =>
//                     state.update(|state| state.points.truncate(total)),
//                 None => (),
//             }
//         }
//
//         // Check whether or not we need to centre the points.
//         match maybe_shift_to_centre_from {
//             Some(original) => {
//                 let xy = rect.xy();
//                 let difference = vec2_sub(xy, original);
//                 update_points(state, points.into_iter().map(|point| vec2_add(point, difference)))
//             },
//             None => update_points(state, points),
//         }
//
//         let kind = match *style {
//             Style::Fill(_) => Kind::Fill,
//             Style::Outline(_) => Kind::Outline,
//         };
//
//         if state.kind != kind {
//             state.update(|state| state.kind = kind);
//         }
//     }
//
// }
// */
//
// /// Triangulate the polygon given as a list of `Point`s describing its sides.
// ///
// /// Returns `None` if the given iterator yields less than two points.
// pub fn triangles<I>(points: I) -> Option<Triangles<I::IntoIter>>
//     where I: IntoIterator<Item=Point>,
// {
//     let mut points = points.into_iter();
//     let first = match points.next() {
//         Some(p) => p,
//         None => return None,
//     };
//     let prev = match points.next() {
//         Some(p) => p,
//         None => return None,
//     };
//     Some(Triangles {
//         first,
//         prev,
//         points,
//     })
// }
//
// impl<I> Iterator for Triangles<I>
//     where I: Iterator<Item=Point>,
// {
//     type Item = Triangle<Point>;
//     fn next(&mut self) -> Option<Self::Item> {
//         self.points.next().map(|point| {
//             let t = Triangle([self.first, self.prev, point]);
//             self.prev = point;
//             t
//         })
//     }
// }
//
//
// /*/// The function to use for picking whether a given point is over the polygon.
// pub fn is_over_widget(widget: &graph::Container, point: Point, _: &Theme) -> widget::IsOver {
//     widget
//         .state_and_style::<State, Style>()
//         .map(|widget| is_over(widget.state.points.iter().cloned(), point))
//         .unwrap_or_else(|| widget.rect.is_over(point))
//         .into()
// }*/
