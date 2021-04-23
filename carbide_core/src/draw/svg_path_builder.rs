use lyon::algorithms::math::{Angle, Point, Vector};
use lyon::algorithms::path::{ArcFlags, Path};
use lyon::algorithms::path::builder::Build;

#[derive(Clone)]
pub struct SVGPathBuilder {
    actions: Vec<SVGBuildAction>
}

impl SVGPathBuilder {
    pub fn new() -> Self {
        SVGPathBuilder {
            actions: vec![]
        }
    }
}

impl Build for SVGPathBuilder {
    type PathType = Path;

    fn build(self) -> Path {
        let mut builder = Path::builder().with_svg();

        for action in &self.actions {
            match action {
                SVGBuildAction::MoveTo { to } => {
                    builder.move_to(*to);
                }
                SVGBuildAction::Close => {
                    builder.close();
                }
                SVGBuildAction::LineTo { to } => {
                    builder.line_to(*to);
                }
                SVGBuildAction::QuadraticBezierTo { ctrl, to } => {
                    builder.quadratic_bezier_to(*ctrl, *to);
                }
                SVGBuildAction::CubicBezierTo { ctrl1, ctrl2, to } => {
                    builder.cubic_bezier_to(*ctrl1, *ctrl2, *to);
                }
            }
        }

        builder.build()
    }
}

impl lyon::path::builder::SvgPathBuilder for SVGPathBuilder {
    fn move_to(&mut self, to: Point) {
        self.actions.push(SVGBuildAction::MoveTo {to});
    }

    fn close(&mut self) {
        self.actions.push(SVGBuildAction::Close);
    }

    fn line_to(&mut self, to: Point) {
        self.actions.push(SVGBuildAction::LineTo {to});
    }

    fn quadratic_bezier_to(&mut self, ctrl: Point, to: Point) {
        self.actions.push(SVGBuildAction::QuadraticBezierTo {ctrl, to});
    }

    fn cubic_bezier_to(&mut self, ctrl1: Point, ctrl2: Point, to: Point) {
        self.actions.push(SVGBuildAction::CubicBezierTo {ctrl1, ctrl2, to});
    }

    fn relative_move_to(&mut self, to: Vector) {
        todo!()
    }

    fn relative_line_to(&mut self, to: Vector) {
        todo!()
    }

    fn relative_quadratic_bezier_to(&mut self, ctrl: Vector, to: Vector) {
        todo!()
    }

    fn relative_cubic_bezier_to(&mut self, ctrl1: Vector, ctrl2: Vector, to: Vector) {
        todo!()
    }

    fn smooth_cubic_bezier_to(&mut self, ctrl2: Point, to: Point) {
        todo!()
    }

    fn smooth_relative_cubic_bezier_to(&mut self, ctrl2: Vector, to: Vector) {
        todo!()
    }

    fn smooth_quadratic_bezier_to(&mut self, to: Point) {
        todo!()
    }

    fn smooth_relative_quadratic_bezier_to(&mut self, to: Vector) {
        todo!()
    }

    fn horizontal_line_to(&mut self, x: f32) {
        todo!()
    }

    fn relative_horizontal_line_to(&mut self, dx: f32) {
        todo!()
    }

    fn vertical_line_to(&mut self, y: f32) {
        todo!()
    }

    fn relative_vertical_line_to(&mut self, dy: f32) {
        todo!()
    }

    fn arc_to(&mut self, radii: Vector, x_rotation: Angle, flags: ArcFlags, to: Point) {
        todo!()
    }

    fn relative_arc_to(&mut self, radii: Vector, x_rotation: Angle, flags: ArcFlags, to: Vector) {
        todo!()
    }
}

#[derive(Clone)]
pub enum SVGBuildAction {
    MoveTo {
        to: Point
    },
    Close,
    LineTo {
        to: Point
    },
    QuadraticBezierTo {
        ctrl: Point,
        to: Point
    },
    CubicBezierTo {
        ctrl1: Point,
        ctrl2: Point,
        to: Point
    }

}