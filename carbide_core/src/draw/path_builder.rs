use lyon::algorithms::path::{EndpointId, Path};
use lyon::math::Point;
use lyon::path::Attributes;

#[derive(Clone)]
pub struct PathBuilder {
    actions: Vec<BuildAction>,
    current_end_point: u32,
}

impl PathBuilder {
    pub fn new() -> PathBuilder {
        PathBuilder {
            actions: vec![],
            current_end_point: 0,
        }
    }
}

impl lyon::path::builder::Build for PathBuilder {
    type PathType = Path;

    fn build(self) -> Path {
        let mut builder = Path::builder();
        for action in &self.actions {
            match action {
                BuildAction::Begin { at } => {
                    builder.begin(*at);
                }
                BuildAction::End { close } => {
                    builder.end(*close);
                }
                BuildAction::LineTo { to } => {
                    builder.line_to(*to);
                }
                BuildAction::QuadraticBezierTo { ctrl, to } => {
                    builder.quadratic_bezier_to(*ctrl, *to);
                }
                BuildAction::CubicBezierTo { ctrl1, ctrl2, to } => {
                    builder.cubic_bezier_to(*ctrl1, *ctrl2, *to);
                }
            }
        }
        builder.build()
    }
}

impl lyon::path::builder::PathBuilder for PathBuilder {
    fn num_attributes(&self) -> usize {
        0
    }

    fn begin(&mut self, at: Point, attr: Attributes) -> EndpointId {
        self.actions.push(BuildAction::Begin { at });
        EndpointId(self.current_end_point)
    }

    fn end(&mut self, close: bool) {
        self.actions.push(BuildAction::End { close })
    }

    fn line_to(&mut self, to: Point, attr: Attributes) -> EndpointId {
        self.actions.push(BuildAction::LineTo { to });
        let id = EndpointId(self.current_end_point);
        self.current_end_point += 1;

        id
    }

    fn quadratic_bezier_to(&mut self, ctrl: Point, to: Point, attr: Attributes) -> EndpointId {
        self.actions
            .push(BuildAction::QuadraticBezierTo { ctrl, to });
        self.current_end_point += 1;
        let id = EndpointId(self.current_end_point);
        self.current_end_point += 1;

        id
    }

    fn cubic_bezier_to(&mut self, ctrl1: Point, ctrl2: Point, to: Point, attr: Attributes) -> EndpointId {
        self.actions
            .push(BuildAction::CubicBezierTo { ctrl1, ctrl2, to });
        self.current_end_point += 2;
        let id = EndpointId(self.current_end_point);
        self.current_end_point += 1;

        id
    }
}

#[derive(Clone)]
pub enum BuildAction {
    Begin {
        at: Point,
    },
    End {
        close: bool,
    },
    LineTo {
        to: Point,
    },
    QuadraticBezierTo {
        ctrl: Point,
        to: Point,
    },
    CubicBezierTo {
        ctrl1: Point,
        ctrl2: Point,
        to: Point,
    },
}
