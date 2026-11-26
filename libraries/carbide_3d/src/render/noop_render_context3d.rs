use carbide::color::Color;
use carbide::environment::Environment;
use carbide::math::{Deg, Matrix4, Vector3};
use carbide::render::Layer;
use crate::camera::CameraSpec;
use crate::material::Material;
use crate::{InnerImageContext3d, Mesh, NoopImageContext3d};
use crate::render::{InnerRenderContext3d, LightShadow, MeshSettings, ViewSpec};

/// The render context of a scene without a 3D renderer, used until a
/// [`ContextFactory3d`](crate::render::ContextFactory3d) is found in the environment. It draws
/// nothing, so the scene's layer stays empty and transparent.
#[derive(Clone, Debug)]
pub struct NoopRenderContext3d;

impl InnerRenderContext3d for NoopRenderContext3d {
    fn start(&mut self) {
    }

    fn render(&mut self, _layer: Layer, _camera: CameraSpec, _view: ViewSpec, _env: &mut Environment) {
    }

    fn mesh(&mut self, _mesh: &Mesh, _settings: MeshSettings) {
    }

    fn material(&mut self, _material: &Material, _env: &mut Environment) {
    }

    fn pop_material(&mut self) {
    }

    fn transform(&mut self, _transform: &Matrix4<f32>) {
    }

    fn pop_transform(&mut self) {
    }

    fn directional(&mut self, _color: Color, _intensity: f32, _direction: Vector3<f32>, _shadow: Option<LightShadow>) {
    }

    fn point(&mut self, _color: Color, _intensity: f32, _range: Option<f32>) {
    }

    fn spot(&mut self, _color: Color, _intensity: f32, _range: Option<f32>, _inner: Deg<f32>, _outer: Deg<f32>, _shadow: Option<LightShadow>) {
    }

    fn hemisphere(&mut self, _sky: Color, _ground: Color, _intensity: f32) {
    }

    fn image_context(&mut self) -> &mut dyn InnerImageContext3d {
        // `NoopImageContext3d` is zero-sized, so leaking a box of it allocates nothing.
        Box::leak(Box::new(NoopImageContext3d))
    }
}

#[cfg(test)]
mod tests {
    use carbide::draw::ImageId;
    use carbide::environment::Environment;
    use crate::render::{InnerRenderContext3d, NoopRenderContext3d};

    #[test]
    fn image_context_has_no_textures() {
        let mut env = Environment::new();
        let mut context = NoopRenderContext3d;
        let image_context = context.image_context();
        let id = ImageId::new("missing.png");

        assert!(!image_context.texture_exist(&id, &mut env));
        assert_eq!(image_context.texture_dimensions(&id, &mut env), None);
    }
}