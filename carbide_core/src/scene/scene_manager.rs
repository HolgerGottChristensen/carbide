use smallvec::SmallVec;
use carbide::draw::Dimension;
use carbide::environment::Key;
use carbide::scene::{AnyScene, Scene, SceneId};
use crate::draw::Scalar;
use crate::environment::EnvironmentStack;

#[derive(Debug, Clone)]
pub struct SceneManager {
    scale_factor: Scalar,
    physical_dimensions: Dimension,
    close: bool,
    add_scenes: SmallVec<[Box<dyn AnyScene>; 1]>,
    close_scenes: SmallVec<[SceneId; 1]>,
}

impl SceneManager {
    pub fn new(scale_factor: Scalar, physical_dimensions: Dimension) -> SceneManager {
        SceneManager {
            scale_factor,
            physical_dimensions,
            close: false,
            add_scenes: Default::default(),
            close_scenes: Default::default(),
        }
    }

    pub fn physical_dimensions(&self) -> Dimension {
        self.physical_dimensions
    }

    pub fn dimensions(&self) -> Dimension {
        self.physical_dimensions / self.scale_factor
    }

    pub fn scale_factor(&self) -> Scalar {
        self.scale_factor
    }

    pub fn add_sub_scene(&mut self, scene: impl Scene) {
        self.add_scenes.push(Box::new(scene))
    }

    pub fn scenes_to_add(&mut self) -> &mut SmallVec<[Box<dyn AnyScene>; 1]> {
        &mut self.add_scenes
    }

    pub fn close_sub_scene(&mut self, id: SceneId) {
        self.close_scenes.push(id);
    }

    pub fn sub_scenes_to_close(&mut self) -> &[SceneId] {
        &self.close_scenes
    }

    pub fn close(&mut self) {
        self.close = true;
    }

    pub fn close_requested(&self) -> bool {
        self.close
    }

    pub fn get(env_stack: &mut EnvironmentStack, f: impl FnOnce(&mut SceneManager)) {
        if let Some(manager) = env_stack.get_mut::<SceneManager>() {
            f(manager)
        }
    }
}

impl Key for SceneManager {
    type Value = SceneManager;
}