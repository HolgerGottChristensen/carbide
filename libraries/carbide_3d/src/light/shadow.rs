use carbide::environment::Environment;

use crate::render::LightShadow;
use crate::shadows::CastsShadows;

/// A light's shadow settings: how it casts shadows when it does. Whether it does comes from the
/// environment ([`CastsShadows`]), on where nothing is set.
#[derive(Debug, Clone)]
pub(crate) struct ShadowSettings {
    pub(crate) resolution: u32,
    pub(crate) bias: f32,
    pub(crate) distance: f32,
}

impl ShadowSettings {
    pub(crate) fn new() -> ShadowSettings {
        ShadowSettings {
            resolution: LightShadow::DEFAULT.resolution,
            bias: LightShadow::DEFAULT.bias,
            distance: LightShadow::DEFAULT.distance,
        }
    }

    /// The settings to render with, or `None` when the light doesn't cast shadows here.
    pub(crate) fn spec(&self, env: &Environment) -> Option<LightShadow> {
        if !env.get::<CastsShadows>().copied().unwrap_or(true) {
            return None;
        }

        Some(LightShadow {
            resolution: self.resolution.max(1),
            bias: self.bias,
            distance: self.distance,
        })
    }
}

