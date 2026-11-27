use carbide::color::{Color, TRANSPARENT};
use carbide::impl_state_value;
use carbide::environment::{Environment, EnvironmentColor, EnvironmentKeyable};
use carbide::state::{AnyReadState, ConvertIntoRead, EnvMap1, Map1, RMap1};

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

/// What a view shows behind everything it draws.
///
/// A [`Color`] converts to it, so a view's `background` takes colours, including the theme's
/// `EnvironmentColor`s, as well.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Background {
    /// A colour. Transparent shows the UI behind the view.
    Color(Color),
    /// The scene's environment image (see `Scene3d::environment`), as seen in each pixel's
    /// direction, at the environment's intensity, exposed and tone mapped as the scene. `blur`
    /// from 0, sharp, to 1, as blurred as the roughest reflection: a blur of 0.5 looks as a
    /// surface of roughness 0.5 reflects it. Transparent while the scene has no environment, or
    /// its image isn't loaded. Seen through an orthographic camera, every pixel looks the same
    /// way, so it is one colour.
    Environment { blur: f32 },
}

impl Default for Background {
    fn default() -> Self {
        Background::Color(TRANSPARENT)
    }
}

impl_state_value!(Background);

impl ConvertIntoRead<Background> for Color {
    type Output<G: AnyReadState<T=Color> + Clone> = RMap1<fn(&Color)->Background, Color, Background, G>;

    fn convert<F: AnyReadState<T=Color> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |color| Background::Color(*color))
    }
}

/// The theme's colours, as a background, following the theme.
impl ConvertIntoRead<Background> for EnvironmentColor {
    type Output<G: AnyReadState<T=Self> + Clone> = EnvMap1<fn(&mut Environment, &EnvironmentColor)->Background, EnvironmentColor, Background, G>;

    fn convert<F: AnyReadState<T=EnvironmentColor> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map_env(f, |env, value| Background::Color(value.get(env).unwrap_or_default()))
    }
}

/// How one view looks at its scene, apart from its camera: what a render context needs besides
/// the scene's nodes.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ViewSpec {
    /// What is shown behind everything drawn.
    pub background: Background,
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
            background: Background::default(),
            exposure: 1.0,
            tonemapping: Tonemapping::default(),
            picking: false,
        }
    }
}
