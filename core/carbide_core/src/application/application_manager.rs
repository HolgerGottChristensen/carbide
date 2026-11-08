use std::sync::atomic::{AtomicU32, Ordering};
use smallvec::SmallVec;
use crate::scene::{Scene, SceneId};
use crate::environment::{Environment, EnvironmentKey, Exclusive};
use crate::scene::{AnyScene, SceneSequence};

static FRAME: AtomicU32 = AtomicU32::new(0);

#[derive(Debug)]
pub struct ApplicationManager {
    application_frame: u32,
    close: bool,
    dismiss_scenes: SmallVec<[SceneId; 1]>,
    add_scenes: SmallVec<[Box<dyn AnyScene>; 1]>,
}

impl ApplicationManager {
    pub fn new() -> ApplicationManager {
        ApplicationManager {
            application_frame: 0,
            close: false,
            dismiss_scenes: Default::default(),
            add_scenes: Default::default(),
        }
    }

    pub fn application_frame() -> u32 {
        FRAME.load(Ordering::Relaxed)
    }

    pub fn begin_frame(&mut self) {
        FRAME.fetch_add(1, Ordering::Relaxed);
        self.add_scenes.clear();
        self.dismiss_scenes.clear();
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

    pub fn dismiss_scene(&mut self, id: SceneId) {
        self.dismiss_scenes.push(id);
    }

    pub fn scenes_to_add(&mut self) -> &mut SmallVec<[Box<dyn AnyScene>; 1]> {
        &mut self.add_scenes
    }

    pub fn scenes_to_dismiss(&self) -> &SmallVec<[SceneId; 1]> {
        &self.dismiss_scenes
    }

    /// Open `url` (or a local file path) with whatever application the OS has registered for
    /// it — the default browser for an `http(s)://` url, the default mail client for a
    /// `mailto:` link, the default viewer for a file path, etc. Runs on a background thread so
    /// it never blocks the calling (UI) thread waiting for the launched application to exit.
    #[cfg(feature = "open-url")]
    pub fn open_url(&self, url: impl AsRef<str>) {
        let url = url.as_ref().to_string();
        std::thread::spawn(move || {
            if let Err(e) = open::that(&url) {
                eprintln!("carbide: failed to open {}: {}", url, e);
            }
        });
    }

    pub fn get(env: &mut Environment, f: impl FnOnce(&mut ApplicationManager)) {
        if let Some(manager) = env.get_mut::<ApplicationManager>() {
            f(manager)
        }
    }
}

impl Default for ApplicationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentKey for ApplicationManager {
    type Value = ApplicationManager;
    type Access = Exclusive;
}