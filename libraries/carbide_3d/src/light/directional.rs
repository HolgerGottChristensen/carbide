use crate::node3d::{AnyNode3d, CommonNode3d, NodeId};
use carbide::color::{Color, WHITE};
use carbide::environment::Environment;
use carbide::math::{InnerSpace, Matrix4, SquareMatrix, Vector3, Zero};
use carbide::state::{IntoReadState, ReadState, StateSync};
use crate::light::shadow::ShadowSettings;
use crate::render::{Render3d, RenderContext3d};

/// A light shining in one direction everywhere, like the sun: its direction is the way it shines,
/// in the space of the node, so a light in a turned group turns with it. Its intensity is in lux.
///
/// It casts shadows unless [`CastsShadows`](crate::CastsShadows) is set to false around it, for
/// example with `.casts_shadows(false)` on it, a group around it, or the `Scene3d`. Its shadow map
/// covers a distance in front of the view's camera.
#[derive(Clone, Debug)]
pub struct DirectionalLight<D, C, I>
    where
        D: ReadState<T=Vector3<f32>>,
        C: ReadState<T=Color>,
        I: ReadState<T=f32>
{
    id: NodeId,
    /// Color of the light. The alpha component is not used.
    color: C,
    /// Constant multiplier for the light.
    intensity: I,
    /// Direction of the sun.
    direction: D,
    shadow: ShadowSettings,
}

impl DirectionalLight<Vector3<f32>, Color, f32> {
    pub fn new<D: IntoReadState<Vector3<f32>>>(direction: D) -> DirectionalLight<D::Output, Color, f32> {
        DirectionalLight {
            id: Default::default(),
            color: WHITE,
            intensity: 10.0,
            direction: direction.into_read_state(),
            shadow: ShadowSettings::new(),
        }
    }
}

impl<D: ReadState<T=Vector3<f32>>, C: ReadState<T=Color>, I: ReadState<T=f32>> DirectionalLight<D, C, I> {
    pub fn color<C2: IntoReadState<Color>>(self, color: C2) -> DirectionalLight<D, C2::Output, I> {
        DirectionalLight {
            id: self.id,
            color: color.into_read_state(),
            intensity: self.intensity,
            direction: self.direction,
            shadow: self.shadow,
        }
    }

    pub fn intensity<I2: IntoReadState<f32>>(self, intensity: I2) -> DirectionalLight<D, C, I2::Output> {
        DirectionalLight {
            id: self.id,
            color: self.color,
            intensity: intensity.into_read_state(),
            direction: self.direction,
            shadow: self.shadow,
        }
    }

    /// The shadow map's width and height in texels. 2048 by default.
    pub fn shadow_resolution(mut self, resolution: u32) -> Self {
        self.shadow.resolution = resolution;
        self
    }

    /// How far, in metres, surfaces are moved towards the light before being tested, so they
    /// don't shadow themselves. 2 cm by default.
    pub fn shadow_bias(mut self, bias: f32) -> Self {
        self.shadow.bias = bias;
        self
    }

    /// The distance in front of the view's camera, in metres, that the shadow map covers. 10 m by
    /// default.
    pub fn shadow_distance(mut self, distance: f32) -> Self {
        self.shadow.distance = distance;
        self
    }
}

impl<D: ReadState<T=Vector3<f32>>, C: ReadState<T=Color>, I: ReadState<T=f32>> StateSync for DirectionalLight<D, C, I> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.direction.sync(env) | self.color.sync(env) | self.intensity.sync(env)
    }
}

impl<D: ReadState<T=Vector3<f32>>, C: ReadState<T=Color>, I: ReadState<T=f32>> Render3d for DirectionalLight<D, C, I> {
    fn render(&mut self, context: &mut RenderContext3d) {
        self.sync(context.env);
        //let new_direction = (Matrix4::from(Euler::new(Deg(0.0), Deg(0.0), Deg(1.0))) * Vector4::new(self.direction.x, self.direction.y, self.direction.z, 1.0));

        //self.direction = Vector3::new(new_direction.x, new_direction.y, new_direction.z);
        // A light that gives no light costs nothing: no lighting, and no shadow map.
        if !self.direction.value().is_zero() && *self.intensity.value() > 0.0 {
            let color = *self.color.value();//.with_opacity(1.0);
            let intensity = *self.intensity.value();
            let direction = self.direction.value().normalize();
            let shadow = self.shadow.spec(context.env);
            context.directional(color, intensity, direction, shadow)
        }
    }
}

impl<D: ReadState<T=Vector3<f32>>, C: ReadState<T=Color>, I: ReadState<T=f32>> CommonNode3d for DirectionalLight<D, C, I> {
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