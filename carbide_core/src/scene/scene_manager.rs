use carbide::draw::Dimension;
use carbide::environment::Key;
use crate::draw::Scalar;

#[derive(Debug, Clone)]
pub struct SceneManager {
    scale_factor: Scalar,
    physical_dimensions: Dimension,
    close: bool
}

impl SceneManager {
    pub fn new(scale_factor: Scalar, physical_dimensions: Dimension) -> SceneManager {
        SceneManager {
            scale_factor,
            physical_dimensions,
            close: false,
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

    pub fn close(&mut self) {
        self.close = true;
    }
}

impl Key for SceneManager {
    type Value = SceneManager;
}