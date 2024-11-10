use smallvec::SmallVec;
use carbide::scene::SceneId;
use crate::environment::{EnvironmentStack, Key};

#[derive(Debug)]
pub struct ApplicationManager {
    close: bool,
    close_scenes: SmallVec<[SceneId; 1]>,
}

impl ApplicationManager {
    pub fn new() -> ApplicationManager {
        ApplicationManager {
            close: false,
            close_scenes: Default::default(),
        }
    }

    pub fn close(&mut self) {
        self.close = true;
    }

    pub fn close_requested(&self) -> bool {
        self.close
    }

    pub fn close_scene(&mut self, id: SceneId) {
        self.close_scenes.push(id);
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