use carbide::color::{Color, WHITE};
use carbide::environment::Environment;
use carbide::math::{Deg, Matrix4, SquareMatrix};
use carbide::state::{AnyReadState, IntoReadState, Map1, ReadState, StateSync};

use crate::node3d::{AnyNode3d, CommonNode3d, NodeId};
use crate::light::shadow::ShadowSettings;
use crate::render::{Render3d, RenderContext3d};

/// A light shining from a point in a cone, at its node's origin, along its node's -Z (forward, where
/// the default camera looks, as in glTF): place it with `.position()` and aim it with
/// `.rotation()`, as an [`Object`](crate::Object).
///
/// Inside the inner angle from its axis it is at full brightness, outside the outer angle it is
/// dark, and in between it fades smoothly. Its intensity is in candela, and it falls off with
/// distance as a [`PointLight`](crate::light::PointLight) does.
#[derive(Debug, Clone)]
pub struct SpotLight {
    id: NodeId,
    color: Box<dyn AnyReadState<T=Color>>,
    intensity: Box<dyn AnyReadState<T=f32>>,
    range: Box<dyn AnyReadState<T=Option<f32>>>,
    inner: Box<dyn AnyReadState<T=Deg<f32>>>,
    outer: Box<dyn AnyReadState<T=Deg<f32>>>,
    shadow: ShadowSettings,
}

impl SpotLight {
    /// A white light of 10 candela, with no range, at full brightness within `inner` of its axis
    /// and dark beyond `outer`.
    pub fn new<I: IntoReadState<Deg<f32>>, O: IntoReadState<Deg<f32>>>(inner: I, outer: O) -> SpotLight {
        SpotLight {
            id: NodeId::new(),
            color: Box::new(WHITE),
            intensity: Box::new(10.0f32),
            range: Box::new(None::<f32>),
            inner: Box::new(inner.into_read_state()),
            outer: Box::new(outer.into_read_state()),
            shadow: ShadowSettings::new(),
        }
    }

    /// The light's colour.
    pub fn color<C: IntoReadState<Color>>(mut self, color: C) -> SpotLight {
        self.color = Box::new(color.into_read_state());
        self
    }

    /// The light's intensity inside the inner angle, in candela.
    pub fn intensity<I: IntoReadState<f32>>(mut self, intensity: I) -> SpotLight {
        self.intensity = Box::new(intensity.into_read_state());
        self
    }

    /// The distance, in metres, at which the light has faded to nothing. Without one, it follows
    /// the inverse square law at every distance.
    pub fn range<R: IntoReadState<f32>>(mut self, range: R) -> SpotLight {
        self.range = Box::new(Map1::read_map(range.into_read_state(), |range: &f32| Some(*range)));
        self
    }

    /// The shadow map's width and height in texels. 2048 by default.
    pub fn shadow_resolution(mut self, resolution: u32) -> SpotLight {
        self.shadow.resolution = resolution;
        self
    }

    /// How far, in metres, surfaces are moved towards the light before being tested, so they
    /// don't shadow themselves. 2 cm by default.
    pub fn shadow_bias(mut self, bias: f32) -> SpotLight {
        self.shadow.bias = bias;
        self
    }

    /// The angles from the light's axis: full brightness within `inner`, dark beyond `outer`.
    pub fn angles<I: IntoReadState<Deg<f32>>, O: IntoReadState<Deg<f32>>>(mut self, inner: I, outer: O) -> SpotLight {
        self.inner = Box::new(inner.into_read_state());
        self.outer = Box::new(outer.into_read_state());
        self
    }
}

impl StateSync for SpotLight {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.color.sync(env) | self.intensity.sync(env) | self.range.sync(env) | self.inner.sync(env) | self.outer.sync(env)
    }
}

impl Render3d for SpotLight {
    fn render(&mut self, context: &mut RenderContext3d) {
        self.sync(context.env);
        // A light that gives no light costs nothing: no lighting, and no shadow map.
        if *self.intensity.value() <= 0.0 {
            return;
        }
        context.spot(*self.color.value(), *self.intensity.value(), *self.range.value(), *self.inner.value(), *self.outer.value(), self.shadow.spec(context.env));
    }
}

impl CommonNode3d for SpotLight {
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
