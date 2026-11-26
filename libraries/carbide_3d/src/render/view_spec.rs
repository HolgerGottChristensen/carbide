use carbide::color::{Color, TRANSPARENT};
use carbide::impl_state_value;

/// How a view's lit colours become the colours on screen, after exposure.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Tonemapping {
    /// Khronos PBR Neutral: keeps base colours as they are under normal lighting, and compresses
    /// only what would clip, desaturating it towards white.
    #[default]
    PbrNeutral,
    /// The ACES filmic curve (Narkowicz's fit): more contrast, and shifts hues in bright areas.
    Aces,
    /// No tone mapping: colours are clamped, so they are exact up to the brightest displayable
    /// value. For diagrams and charts.
    None,
}

impl_state_value!(Tonemapping);

/// How one view looks at its scene, apart from its camera: what a render context needs besides
/// the scene's nodes.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ViewSpec {
    /// The colour behind everything drawn. Transparent shows the UI behind the view.
    pub background: Color,
    /// What lit colours are multiplied by before tone mapping.
    pub exposure: f32,
    pub tonemapping: Tonemapping,
    /// Whether to render a picking buffer with the frame, for [`pick`](crate::render::InnerRenderContext3d::pick)
    /// to read until the next frame.
    pub picking: bool,
}

impl Default for ViewSpec {
    fn default() -> Self {
        ViewSpec {
            background: TRANSPARENT,
            exposure: 1.0,
            tonemapping: Tonemapping::default(),
            picking: false,
        }
    }
}
