//! Which way a controller's values grow. Bar and line controllers stand upright by default, the
//! categories along the x scale and the values up the y scale, and can be laid down with
//! `horizontal`, the categories down the y scale and the values along the x scale. The data is
//! read the same way both times: a point's x is its category and its y its value.
//!
//! Drawing code that depends on which way is which (monotone curves, steps, fills down to a
//! baseline, the edge of a stacked area) works in the index frame: positions whose x is along the
//! categories and whose y is along the values. For upright controllers it is the screen itself;
//! for lying ones it is the screen with x and y swapped, so the same code draws both, and
//! `Oriented` swaps them back on the way to the canvas.

use carbide::draw::{Dimension, Position, Rect, Scalar};
use crate::DataSetOptions;
use crate::controller::line_path::PathSink;
use crate::scale::{Axis, Places, PolarRole, Scale, ScaleBounds, ScaleId};

/// The direction a controller's values grow in.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub(crate) enum Orientation {
    /// Values grow up the y scale, and the categories run along the x scale.
    #[default]
    Vertical,
    /// Values grow along the x scale, and the categories run down the y scale.
    Horizontal,
}

impl Orientation {
    /// The axis the categories run along.
    pub(crate) fn index_axis(self) -> Axis {
        match self {
            Orientation::Vertical => Axis::Horizontal,
            Orientation::Horizontal => Axis::Vertical,
        }
    }

    /// The axis the values grow along.
    pub(crate) fn value_axis(self) -> Axis {
        match self {
            Orientation::Vertical => Axis::Vertical,
            Orientation::Horizontal => Axis::Horizontal,
        }
    }

    /// The ids of the dataset's scales the categories and the values go on: its x and y scales
    /// when upright, its y and x scales when lying down.
    pub(crate) fn index_and_value_scales(self, options: &DataSetOptions) -> (&ScaleId, &ScaleId) {
        match self {
            Orientation::Vertical => (&options.x_scale, &options.y_scale),
            Orientation::Horizontal => (&options.y_scale, &options.x_scale),
        }
    }

    /// How a dataset with `options` reads its points: as this orientation says, unless it is on
    /// two radial scales. A polar dataset keeps its x on its x scale and its y on its y scale, and
    /// the orientation swaps the scales' roles instead (see `polar_roles`), so the same scales
    /// and labels draw a radar upright and categories out along the radius lying down.
    pub(crate) fn for_dataset(self, options: &DataSetOptions, places: &dyn Places) -> Orientation {
        if is_polar(options, places) {
            Orientation::Vertical
        } else {
            self
        }
    }

    /// The roles of a polar dataset's x and y scales: upright, the x scale is the angle and the
    /// y scale the radius, so the categories run around the circle; lying down, the other way
    /// round, so they run out along the radius.
    pub(crate) fn polar_roles(self) -> (PolarRole, PolarRole) {
        match self {
            Orientation::Vertical => (PolarRole::Angle, PolarRole::Radius),
            Orientation::Horizontal => (PolarRole::Radius, PolarRole::Angle),
        }
    }

    /// Asks for the roles of a polar dataset's scales (see `polar_roles`), when the dataset with
    /// `options` is on two radial scales.
    pub(crate) fn request_polar_roles(self, options: &DataSetOptions, bounds: &ScaleBounds) {
        if bounds.is_radial(&options.x_scale) && bounds.is_radial(&options.y_scale) {
            let (x, y) = self.polar_roles();
            bounds.request_polar_role(&options.x_scale, x);
            bounds.request_polar_role(&options.y_scale, y);
        }
    }

    /// Turns a position in the index frame into one on the screen, or back: the two only differ
    /// by swapping x and y, so the same swap goes both ways.
    pub(crate) fn swap(self, position: Position) -> Position {
        match self {
            Orientation::Vertical => position,
            Orientation::Horizontal => Position::new(position.y, position.x),
        }
    }

    /// The chart area in the index frame.
    pub(crate) fn frame_area(self, area: Rect) -> Rect {
        match self {
            Orientation::Vertical => area,
            Orientation::Horizontal => Rect::new(self.swap(area.position), Dimension::new(area.height(), area.width())),
        }
    }
}

/// Whether the dataset with `options` is a polar dataset: on two radial scales.
pub(crate) fn is_polar(options: &DataSetOptions, places: &dyn Places) -> bool {
    places.is_radial(&options.x_scale) && places.is_radial(&options.y_scale)
}

/// Where `value` on `scale` falls on the screen, along `axis`: an x for a horizontal axis, and a y
/// for a vertical one, where a normalized 1 is the top of the chart area.
pub(crate) fn pixel(axis: Axis, scale: &dyn Scale, value: Scalar, area: Rect) -> Scalar {
    normalized_pixel(axis, scale.normalize(value), area)
}

/// Where a normalized position along `axis` falls on the screen: 0 is the left or bottom of the
/// chart area, and 1 its right or top.
pub(crate) fn normalized_pixel(axis: Axis, normalized: Scalar, area: Rect) -> Scalar {
    // `Rect`'s `bottom()` is the screen's top edge (see `scale/axis.rs`).
    match axis {
        Axis::Vertical => area.bottom() + (1.0 - normalized) * area.height(),
        _ => area.left() + normalized * area.width(),
    }
}

/// The chart area's length along `axis`.
pub(crate) fn length(axis: Axis, area: Rect) -> Scalar {
    match axis {
        Axis::Vertical => area.height(),
        _ => area.width(),
    }
}

/// A path traced in the index frame, written to `path` on the screen.
pub(crate) struct Oriented<'a, P: PathSink + ?Sized> {
    pub(crate) path: &'a mut P,
    pub(crate) orientation: Orientation,
}

impl<P: PathSink + ?Sized> PathSink for Oriented<'_, P> {
    fn move_to(&mut self, to: Position) {
        self.path.move_to(self.orientation.swap(to));
    }

    fn line_to(&mut self, to: Position) {
        self.path.line_to(self.orientation.swap(to));
    }

    fn bezier_curve_to(&mut self, control1: Position, control2: Position, to: Position) {
        self.path.bezier_curve_to(self.orientation.swap(control1), self.orientation.swap(control2), self.orientation.swap(to));
    }
}

#[cfg(test)]
mod tests {
    use carbide::draw::{Dimension, Position, Rect};
    use crate::controller::line_path::PathSink;
    use super::{Oriented, Orientation};

    /// The index frame of a lying controller is the screen with x and y swapped, the chart area
    /// included, and a path traced in it reaches the screen swapped back.
    #[test]
    fn the_horizontal_frame_swaps_x_and_y() {
        let area = Rect::new(Position::new(10.0, 20.0), Dimension::new(300.0, 200.0));
        let frame = Orientation::Horizontal.frame_area(area);

        assert_eq!((frame.left(), frame.bottom(), frame.width(), frame.height()), (20.0, 10.0, 200.0, 300.0));
        assert_eq!(Orientation::Vertical.frame_area(area), area, "upright, the frame is the screen");

        let mut traced = Traced(vec![]);
        let mut path = Oriented { path: &mut traced, orientation: Orientation::Horizontal };
        path.move_to(Position::new(1.0, 2.0));
        path.line_to(Position::new(3.0, 4.0));
        path.bezier_curve_to(Position::new(5.0, 6.0), Position::new(7.0, 8.0), Position::new(9.0, 10.0));

        assert_eq!(traced.0, vec![
            Position::new(2.0, 1.0),
            Position::new(4.0, 3.0),
            Position::new(6.0, 5.0), Position::new(8.0, 7.0), Position::new(10.0, 9.0),
        ]);
    }

    /// Records the positions a path is traced through.
    struct Traced(Vec<Position>);

    impl PathSink for Traced {
        fn move_to(&mut self, to: Position) {
            self.0.push(to);
        }

        fn line_to(&mut self, to: Position) {
            self.0.push(to);
        }

        fn bezier_curve_to(&mut self, control1: Position, control2: Position, to: Position) {
            self.0.extend([control1, control2, to]);
        }
    }
}
