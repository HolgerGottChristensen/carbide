use crate::draw::Position;
use crate::environment::{EnvironmentStack, Key};

#[derive(Debug, Copy, Clone)]
pub struct MousePositionKey;

impl Key for MousePositionKey {
    type Value = Position;
}

pub trait MousePositionEnvironmentExt {
    fn mouse_position(&self) -> Position;
}

impl MousePositionEnvironmentExt for EnvironmentStack<'_> {
    fn mouse_position(&self) -> Position {
        self.get::<MousePositionKey>().cloned().unwrap_or_default()
    }
}