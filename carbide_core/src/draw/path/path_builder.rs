use carbide::draw::{Angle, Dimension, Position};
use crate::draw::path::path::Path;
use crate::draw::path::path_instruction::PathInstruction;

pub struct PathBuilder {
    path: Path
}

impl PathBuilder {
    pub fn new() -> PathBuilder {
        PathBuilder { path: Path { instructions: vec![] } }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn move_to(&mut self, to: Position) {
        self.path.instructions.push(PathInstruction::MoveTo { to });
    }

    pub fn close(&mut self) {
        self.path.instructions.push(PathInstruction::Close)
    }

    pub fn line_to(&mut self, to: Position) {
        self.path.instructions.push(PathInstruction::LineTo { to });
    }

    pub fn quadratic_bezier_to(&mut self, ctrl: Position, to: Position) {
        self.path.instructions.push(PathInstruction::QuadraticBezierTo { ctrl, to });
    }

    pub fn cubic_bezier_to(&mut self, ctrl1: Position, ctrl2: Position, to: Position) {
        self.path.instructions.push(PathInstruction::CubicBezierTo { ctrl1, ctrl2, to });
    }

    pub fn arc(&mut self, center: Position, radius: Dimension, start_angle: Angle, end_angle: Angle) {
        self.path.instructions.push(PathInstruction::Arc { center, radius, start_angle, end_angle });
    }

}