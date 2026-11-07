use smallvec::SmallVec;
use crate::draw::Dimension;
use crate::environment::{EnvironmentKey, Exclusive};
use crate::scene::{AnyScene, Scene, SceneId};
use crate::draw::Scalar;
use crate::environment::Environment;

#[derive(Debug)]
pub struct SceneManager {
    scale_factor: Scalar,
    physical_dimensions: Dimension,
    dismiss: bool,
    add_scenes: SmallVec<[Box<dyn AnyScene>; 1]>,
    close_scenes: SmallVec<[SceneId; 1]>,
    screenshot_requests: SmallVec<[oneshot::Sender<image::RgbaImage>; 1]>,
}

impl SceneManager {
    pub fn new(scale_factor: Scalar, physical_dimensions: Dimension) -> SceneManager {
        SceneManager {
            scale_factor,
            physical_dimensions,
            dismiss: false,
            add_scenes: Default::default(),
            close_scenes: Default::default(),
            screenshot_requests: Default::default(),
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

    pub fn dismiss_sub_scene(&mut self, id: SceneId) {
        self.close_scenes.push(id);
    }

    pub fn sub_scenes_to_dismiss(&mut self) -> &[SceneId] {
        &self.close_scenes
    }

    pub fn dismiss(&mut self) {
        self.dismiss = true;
    }

    pub fn dismiss_requested(&self) -> bool {
        self.dismiss
    }

    /// Request a capture of this window's currently-rendered frame (whatever the last
    /// completed render produced — not a fresh one). The receiver resolves as soon as the
    /// backend finishes draining requests for this traversal.
    pub fn take_screenshot(&mut self) -> oneshot::Receiver<image::RgbaImage> {
        let (sender, receiver) = oneshot::channel();
        self.screenshot_requests.push(sender);
        receiver
    }

    /// Convenience: capture the current frame and write it to `path` as a PNG on a background
    /// thread (blocking on the receiver there, so this call itself doesn't block the caller).
    pub fn save_screenshot(&mut self, path: impl AsRef<std::path::Path>) {
        let receiver = self.take_screenshot();
        let path = path.as_ref().to_path_buf();
        std::thread::spawn(move || match receiver.recv() {
            Ok(image) => if let Err(e) = image.save(&path) {
                eprintln!("carbide: failed to save screenshot to {}: {}", path.display(), e);
            },
            Err(_) => eprintln!("carbide: screenshot request was dropped before it could be fulfilled"),
        });
    }

    /// Drained by the backend after rendering a frame; not part of the public API surface
    /// consumers reach for directly (mirrors `scenes_to_add`/`sub_scenes_to_dismiss`).
    pub fn screenshot_requests(&mut self) -> &mut SmallVec<[oneshot::Sender<image::RgbaImage>; 1]> {
        &mut self.screenshot_requests
    }

    pub fn get(env: &mut Environment, f: impl FnOnce(&mut SceneManager)) {
        if let Some(manager) = env.get_mut::<SceneManager>() {
            f(manager)
        }
    }
}

impl EnvironmentKey for SceneManager {
    type Value = SceneManager;
    type Access = Exclusive;
}