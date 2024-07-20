use carbide::render::matrix::Vector3;

pub struct PointLight {
    /// The position of the light in the world.
    pub position: Vector3<f32>,

    /// The color of the light.
    pub color: Vector3<f32>,

    /// The radius of the light.
    pub radius: f32,

    /// Constant multiplier for the light.
    pub intensity: f32,
}