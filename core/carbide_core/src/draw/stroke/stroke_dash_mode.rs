

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum StrokeDashMode {
    /// Fast will only check if the triangle making up the line is in a dash
    Fast,
    /// Pretty will check if triangles that are parts of joins contains dashes.
    /// On thin lines this is not visible.
    Pretty
}