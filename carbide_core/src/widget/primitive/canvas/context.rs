use crate::Point;
use lyon::algorithms::path::Path;
use lyon::lyon_algorithms::path::math::point;

#[derive(Debug, Clone)]
pub struct Context {
    pub actions: Vec<ContextAction>
}


impl Context {
    pub fn to_paths(&self, offset: Point) -> Vec<Path> {

        let offset_point = |p: [f64; 2]| {
            point(p[0] as f32 + offset[0] as f32, p[1] as f32 + offset[1] as f32)
        };

        let mut paths = vec![];

        let mut current_builder = Path::builder();
        let mut current_builder_begun = false;

        for action in &self.actions {
            match action {
                ContextAction::MoveTo(p) => {
                    if !current_builder_begun {
                        current_builder.begin(offset_point(*p));
                        current_builder_begun = true;
                    } else {
                        paths.push(current_builder.build());
                        current_builder = Path::builder();
                        current_builder.begin(offset_point(*p));
                    }
                }
                ContextAction::LineTo(p) => {
                    if !current_builder_begun {
                        current_builder.begin(offset_point([0.0, 0.0]));
                        current_builder_begun = true;
                        current_builder.line_to(offset_point(*p));
                    } else {
                        current_builder.line_to(offset_point(*p));
                    }
                }
                ContextAction::QuadraticBezierTo { ctrl, to } => {
                    if !current_builder_begun {
                        current_builder.begin(offset_point([0.0, 0.0]));
                        current_builder_begun = true;
                        current_builder.quadratic_bezier_to(offset_point(*ctrl), offset_point(*to));
                    } else {
                        current_builder.quadratic_bezier_to(offset_point(*ctrl), offset_point(*to));
                    }
                }
                ContextAction::CubicBezierTo{ ctrl1, ctrl2, to } => {
                    if !current_builder_begun {
                        current_builder.begin(point(0.0,0.0));
                        current_builder_begun = true;
                        current_builder.cubic_bezier_to(offset_point(*ctrl1), offset_point(*ctrl2),offset_point(*to));
                    } else {
                        current_builder.cubic_bezier_to(offset_point(*ctrl1), offset_point(*ctrl2),offset_point(*to));
                    }
                }
                ContextAction::Fill => {
                    if !current_builder_begun {

                    } else {
                        paths.push(current_builder.build());
                        current_builder = Path::builder();
                    }
                }
                ContextAction::Stroke => {
                    if !current_builder_begun {

                    } else {
                        paths.push(current_builder.build());
                        current_builder = Path::builder();
                    }
                }
                ContextAction::Close => {
                    if !current_builder_begun {

                    } else {
                        current_builder.close();
                    }
                }
            }
        }

        if current_builder_begun {
            paths.push(current_builder.build());
        }

        paths

    }
}

#[derive(Debug, Clone)]
pub enum ContextAction {
    MoveTo(Point),
    LineTo(Point),
    QuadraticBezierTo {ctrl: Point, to: Point},
    CubicBezierTo {ctrl1: Point, ctrl2: Point, to: Point},
    Fill,
    Stroke,
    Close
}
