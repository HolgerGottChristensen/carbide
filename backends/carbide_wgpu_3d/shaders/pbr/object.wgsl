struct Object {
    transform: mat4x4<f32>,
    material_index: u32,
    receives_shadows: u32,
    /// The outline's linear colour, and its thickness in physical pixels; 0 for none.
    outline: vec4<f32>,
}
