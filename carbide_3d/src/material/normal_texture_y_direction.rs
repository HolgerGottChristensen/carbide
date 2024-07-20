

/// The direction of the Y (i.e. green) value in the normal maps
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum NormalTextureYDirection {
    /// Right handed. X right, Y up. OpenGL convention.
    Up,
    /// Left handed. X right, Y down. DirectX convention.
    Down,
}

impl Default for NormalTextureYDirection {
    fn default() -> Self {
        Self::Up
    }
}