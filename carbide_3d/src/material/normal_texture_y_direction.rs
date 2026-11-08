

/// The direction of the Y (i.e. green) value in the normal maps
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum NormalTextureYDirection {
    /// Right handed. X right, Y up. OpenGL convention.
    #[default]
    Up,
    /// Left handed. X right, Y down. DirectX convention.
    Down,
}

