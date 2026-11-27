use crate::node3d::NodeId;
use crate::picking::{PickRecorder, Pickable};
use carbide::environment::{EnvironmentKey, Shared};
use crate::outline::outline_in;
use crate::render::MeshSettings;
use crate::shadows::{CastsShadows, ReceivesShadows};
use crate::camera::CameraSpec;
use crate::material::Material;
use crate::mesh::Mesh;
use std::cell::RefCell;
use std::collections::HashSet;
use carbide::draw::{load_encoded_raster, load_local_raster, Color, ImageContext, ImageId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use carbide::environment::Environment;
use carbide::render::Layer;
use carbide::math::{Deg, Matrix4, Vector3};
use crate::render::{InnerRenderContext3d, LightShadow, ViewSpec};

/// Encoded images that come as bytes rather than files, such as those inside a glTF file, by their
/// in-memory ids: decoded the first time a material samples them. Kept for as long as the process
/// runs, so any image store, in any window or test, can load them; freeing them belongs with
/// freeing textures (see the design document).
static ENCODED_IMAGES: Mutex<Option<HashMap<ImageId, Arc<[u8]>>>> = Mutex::new(None);

/// Register `bytes`, an encoded raster image (PNG, JPEG or another format the `image` crate
/// reads), as the contents of `id`: a material that samples `id` decodes them when first drawn.
pub fn register_encoded_image(id: &ImageId, bytes: Arc<[u8]>) {
    ENCODED_IMAGES.lock().unwrap().get_or_insert_with(HashMap::new).insert(id.clone(), bytes);
}

/// The bytes registered for `id`, if any.
fn encoded_image(id: &ImageId) -> Option<Arc<[u8]>> {
    ENCODED_IMAGES.lock().unwrap().as_ref()?.get(id).cloned()
}

thread_local! {
    /// Images that failed to load, so a missing file is reported once instead of every frame.
    static FAILED_TEXTURES: RefCell<HashSet<ImageId>> = RefCell::new(HashSet::new());
}

/// Load the image `id` into the image context, the store 2D images are in, unless it is there
/// already: from bytes registered with [`register_encoded_image`], or from its file. An image that
/// fails to load is reported once and not tried again.
pub(crate) fn load_image(id: &ImageId, image: &mut dyn ImageContext, env: &mut Environment) {
    if FAILED_TEXTURES.with(|failed| failed.borrow().contains(id)) {
        return;
    }

    let loaded = match encoded_image(id) {
        Some(bytes) => load_encoded_raster(id, &bytes, image, env),
        None => load_local_raster(id, image, env),
    };
    if let Err(error) = loaded {
        eprintln!("carbide_3d: {}", error);
        FAILED_TEXTURES.with(|failed| failed.borrow_mut().insert(id.clone()));
    }
}

pub struct RenderContext3d<'a, 'b: 'a> {
    pub(crate) render: &'a mut dyn InnerRenderContext3d,
    /// The image context of the window or harness, shared with 2D: materials load their textures
    /// through it, into the same store 2D images are in.
    pub image: &'a mut dyn ImageContext,
    pub env: &'a mut Environment<'b>,
    /// What this frame records for picking.
    pub(crate) picking: &'a mut PickRecorder,
}

impl<'a, 'b: 'a> RenderContext3d<'a, 'b> {
    pub fn render(&mut self, layer: Layer, camera: CameraSpec, view: ViewSpec) {
        self.render.render(layer, camera, view, self.env)
    }

    pub fn transform<R, F: FnOnce(&mut RenderContext3d)->R>(&mut self, transform: &Matrix4<f32>, f: F) -> R {
        self.render.transform(transform);
        let res = f(self);
        self.render.pop_transform();
        res
    }

    pub fn material<R, F: FnOnce(&mut RenderContext3d)->R>(&mut self, material: &Material, f: F) -> R {
        self.load_textures(material);
        self.render.material(material, self.env);
        let res = f(self);
        self.render.pop_material();
        res
    }

    /// Load the images `material` samples into the image context, the store 2D images are in, unless
    /// they are there already. An image that fails to load is reported once and not tried again.
    fn load_textures(&mut self, material: &Material) {
        let Material::PBR(pbr) = material;

        for id in pbr.textures() {
            load_image(&id, &mut *self.image, self.env);
        }
    }

    /// Render `f` as part of the node `id`, for picking: meshes drawn in it record it in their
    /// path. Objects and pointer handlers call it.
    pub fn pick_node<R, F: FnOnce(&mut RenderContext3d)->R>(&mut self, id: NodeId, f: F) -> R {
        self.picking.push(id);
        let res = f(self);
        self.picking.pop();
        res
    }

    /// Render `f` with the environment key `K` set to `value`.
    pub fn with_env<K: EnvironmentKey<Access=Shared>, R>(&mut self, value: &K::Value, f: impl FnOnce(&mut RenderContext3d) -> R) -> R where K::Value: Clone {
        let render = &mut *self.render;
        let image = &mut *self.image;
        let picking = &mut *self.picking;
        let mut result = None;
        self.env.with::<K>(value, |env| {
            result = Some(f(&mut RenderContext3d { render, image, env, picking }));
        });
        result.unwrap()
    }

    /// Draw `mesh` with the settings in the environment: it casts and receives shadows unless
    /// [`CastsShadows`](crate::CastsShadows) or [`ReceivesShadows`](crate::ReceivesShadows) say
    /// otherwise, and is outlined where [`Outlined`](crate::outline::Outlined) is set.
    pub fn mesh(&mut self, mesh: &Mesh) {
        let settings = MeshSettings {
            casts_shadows: self.env.get::<CastsShadows>().copied().unwrap_or(true),
            receives_shadows: self.env.get::<ReceivesShadows>().copied().unwrap_or(true),
            outline: outline_in(self.env),
            pick_id: if self.env.get::<Pickable>().copied().unwrap_or(false) { self.picking.record() } else { 0 },
        };

        self.render.mesh(mesh, settings)
    }

    /// Draw `mesh` with `settings`, whatever the environment says.
    pub fn mesh_with(&mut self, mesh: &Mesh, settings: MeshSettings) {
        self.render.mesh(mesh, settings)
    }

    pub fn directional(&mut self, color: Color, intensity: f32, direction: Vector3<f32>, shadow: Option<LightShadow>) {
        self.render.directional(color, intensity, direction, shadow)
    }

    pub fn point(&mut self, color: Color, intensity: f32, range: Option<f32>) {
        self.render.point(color, intensity, range)
    }

    pub fn spot(&mut self, color: Color, intensity: f32, range: Option<f32>, inner: Deg<f32>, outer: Deg<f32>, shadow: Option<LightShadow>) {
        self.render.spot(color, intensity, range, inner, outer, shadow)
    }
}



#[cfg(test)]
mod tests {
    use carbide::color::WHITE;
    use carbide::draw::{Dimension, ImageContext, ImageId, ImageMetrics, Texture};
    use carbide::environment::Environment;
    use carbide::render::RenderInstruction;
    use crate::material::normal_texture::NormalTexture;
    use crate::material::normal_texture_y_direction::NormalTextureYDirection;
    use crate::material::pbr_material::PbrMaterial;
    use crate::material::Material;
    use crate::picking::PickRecorder;
    use crate::render::NoopRenderContext3d;
    use super::RenderContext3d;

    /// An image context that records which images were uploaded.
    #[derive(Default)]
    struct RecordingImages {
        uploaded: Vec<(ImageId, u32, u32)>,
    }

    impl ImageContext for RecordingImages {
        fn exist(&self, id: &ImageId, _env: &mut Environment) -> bool {
            self.uploaded.iter().any(|(uploaded, ..)| uploaded == id)
        }

        fn metrics(&self, _id: &ImageId, _env: &mut Environment) -> ImageMetrics {
            ImageMetrics::Unknown
        }

        fn update_texture(&mut self, id: &ImageId, texture: Texture, _env: &mut Environment) -> bool {
            self.uploaded.push((id.clone(), texture.width, texture.height));
            true
        }

        fn update_vector(&mut self, _id: &ImageId, _description: Vec<RenderInstruction>, _size: Dimension, _env: &mut Environment) -> bool {
            false
        }
    }

    fn render_material(material: &Material, images: &mut RecordingImages, env: &mut Environment) {
        let mut render = NoopRenderContext3d;
        let mut picking = PickRecorder::default();
        let mut ctx = RenderContext3d { render: &mut render, image: images, env, picking: &mut picking };
        ctx.material(material, |_| {});
    }

    #[test]
    fn a_material_loads_its_textures_once() {
        let mut env = Environment::new();
        let mut images = RecordingImages::default();
        let rust = ImageId::new("images/rust.png");

        // The same image as albedo and normal map is loaded once.
        let material = Material::PBR(
            PbrMaterial::new()
                .color(rust.clone())
                .normal(NormalTexture::Tricomponent(rust.clone().into(), NormalTextureYDirection::Up)),
        );

        render_material(&material, &mut images, &mut env);
        render_material(&material, &mut images, &mut env);

        assert_eq!(images.uploaded.len(), 1);
        assert_eq!(images.uploaded[0].0, rust);
        assert!(images.uploaded[0].1 > 0 && images.uploaded[0].2 > 0);
    }

    #[test]
    fn a_material_without_textures_loads_nothing() {
        let mut env = Environment::new();
        let mut images = RecordingImages::default();

        render_material(&Material::PBR(PbrMaterial::new().color(WHITE)), &mut images, &mut env);

        assert!(images.uploaded.is_empty());
    }

    #[test]
    fn a_missing_texture_does_not_panic() {
        let mut env = Environment::new();
        let mut images = RecordingImages::default();
        let material = Material::PBR(PbrMaterial::new().color(ImageId::new("images/does_not_exist.png")));

        render_material(&material, &mut images, &mut env);
        render_material(&material, &mut images, &mut env);

        assert!(images.uploaded.is_empty());
    }
}
