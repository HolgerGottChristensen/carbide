use smallvec::SmallVec;
use carbide::scene::{Scene, SceneId};
use crate::environment::{EnvironmentStack, Key};
use crate::scene::{AnyScene, SceneSequence};

#[derive(Debug)]
pub struct ApplicationManager {
    close: bool,
    close_scenes: SmallVec<[SceneId; 1]>,
    add_scenes: SmallVec<[Box<dyn AnyScene>; 1]>,
}

impl ApplicationManager {
    pub fn new() -> ApplicationManager {
        ApplicationManager {
            close: false,
            close_scenes: Default::default(),
            add_scenes: Default::default(),
        }
    }

    pub fn close(&mut self) {
        self.close = true;
    }

    pub fn close_requested(&self) -> bool {
        self.close
    }

    pub fn add_scene(&mut self, scene: impl Scene) {
        self.add_scenes.push(Box::new(scene));
    }

    pub fn add_scenes(&mut self, scenes: impl SceneSequence) {
        for scene in scenes.to_vec() {
            self.add_scenes.push(scene);
        }
    }

    pub fn close_scene(&mut self, id: SceneId) {
        self.close_scenes.push(id);
    }

    pub fn scenes_to_add(&mut self) -> &mut SmallVec<[Box<dyn AnyScene>; 1]> {
        &mut self.add_scenes
    }

    pub fn scenes_to_close(&self) -> &SmallVec<[SceneId; 1]> {
        &self.close_scenes
    }

    pub fn get(env_stack: &mut EnvironmentStack, f: impl FnOnce(&mut ApplicationManager)) {
        if let Some(manager) = env_stack.get_mut::<ApplicationManager>() {
            f(manager)
        }
    }
}

impl Key for ApplicationManager {
    type Value = ApplicationManager;
}