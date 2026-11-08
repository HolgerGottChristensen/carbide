use crate::draw::Position;
use crate::environment::{Environment, EnvironmentKey, Shared};

#[derive(Debug, Copy, Clone)]
pub struct MousePositionKey;

impl EnvironmentKey for MousePositionKey {
    type Value = Position;
    type Access = Shared;
}

pub trait MousePositionEnvironmentExt {
    fn mouse_position(&self) -> Position;
}

impl MousePositionEnvironmentExt for Environment<'_> {
    fn mouse_position(&self) -> Position {
        self.get::<MousePositionKey>().cloned().unwrap_or_default()
    }
}