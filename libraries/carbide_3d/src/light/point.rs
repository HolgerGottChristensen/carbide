use carbide::color::{Color, WHITE};
use carbide::environment::Environment;
use carbide::math::{Matrix4, SquareMatrix};
use carbide::state::{AnyReadState, IntoReadState, Map1, ReadState, StateSync};

use crate::node3d::{AnyNode3d, CommonNode3d, NodeId};
use crate::render::{Render3d, RenderContext3d};

/// A light shining from a point in all directions, at its node's origin: place it with
/// `.position()` and groups, as an [`Object`](crate::Object).
///
/// Its intensity is in candela, as in glTF. It lights a surface `d` metres away, facing it, as a
/// [`DirectionalLight`](crate::light::DirectionalLight) of `intensity / d²` lux would: the inverse
/// square law. With a range, the light fades smoothly to nothing at that distance.
#[derive(Debug, Clone)]
pub struct PointLight {
    id: NodeId,
    color: Box<dyn AnyReadState<T=Color>>,
    intensity: Box<dyn AnyReadState<T=f32>>,
    range: Box<dyn AnyReadState<T=Option<f32>>>,
}

impl PointLight {
    /// A white light of 10 candela, with no range.
    pub fn new() -> PointLight {
        PointLight {
            id: NodeId::new(),
            color: Box::new(WHITE),
            intensity: Box::new(10.0f32),
            range: Box::new(None::<f32>),
        }
    }

    /// The light's colour.
    pub fn color<C: IntoReadState<Color>>(mut self, color: C) -> PointLight {
        self.color = Box::new(color.into_read_state());
        self
    }

    /// The light's intensity, in candela.
    pub fn intensity<I: IntoReadState<f32>>(mut self, intensity: I) -> PointLight {
        self.intensity = Box::new(intensity.into_read_state());
        self
    }

    /// The distance, in metres, at which the light has faded to nothing. Without one, it follows
    /// the inverse square law at every distance.
    pub fn range<R: IntoReadState<f32>>(mut self, range: R) -> PointLight {
        self.range = Box::new(Map1::read_map(range.into_read_state(), |range: &f32| Some(*range)));
        self
    }
}

impl Default for PointLight {
    fn default() -> Self {
        PointLight::new()
    }
}

impl StateSync for PointLight {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.color.sync(env) | self.intensity.sync(env) | self.range.sync(env)
    }
}

impl Render3d for PointLight {
    fn render(&mut self, context: &mut RenderContext3d) {
        self.sync(context.env);
        context.point(*self.color.value(), *self.intensity.value(), *self.range.value());
    }
}

impl CommonNode3d for PointLight {
    fn id(&self) -> NodeId {
        self.id
    }

    fn transform(&self) -> Matrix4<f32> {
        Matrix4::identity()
    }

    fn visible(&self) -> bool {
        true
    }

    fn foreach_child<'a>(&'a self, _f: &mut dyn FnMut(&'a dyn AnyNode3d)) {}

    fn foreach_child_mut<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {}

    fn foreach_child_rev<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {}
}
