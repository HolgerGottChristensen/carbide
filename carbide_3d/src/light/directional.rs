use carbide::color::{Color, WHITE};
use carbide::environment::{Environment, EnvironmentStack};
use carbide::render::matrix::{Deg, Euler, InnerSpace, Matrix4, SquareMatrix, Vector3, Vector4, Zero};
use carbide::state::{IntoReadState, StateSync, ReadState};
use crate::node3d::{AnyNode3d, CommonNode3d, NodeId};
use crate::render3d::Render3d;
use crate::RenderContext3d;

#[derive(Clone, Copy, Debug)]
pub struct DirectionalLight<D, C, I>
    where
        D: ReadState<T=Vector3<f32>>,
        C: ReadState<T=Color>,
        I: ReadState<T=f32>
{
    id: NodeId,
    /// Color of the light. The alpha component is not used.
    color: C,
    /// Resolution of the shadow map cascades (in pix)
    resolution: u16,
    /// Constant multiplier for the light.
    intensity: I,
    /// Direction of the sun.
    direction: D,
    /// Distance from the camera that shadows should be calculated.
    distance: f32,
}

impl DirectionalLight<Vector3<f32>, Color, f32> {
    pub fn new<D: IntoReadState<Vector3<f32>>>(direction: D) -> DirectionalLight<D::Output, Color, f32> {
        DirectionalLight {
            id: Default::default(),
            color: WHITE,
            resolution: 0,
            intensity: 10.0,
            direction: direction.into_read_state(),
            distance: 0.0,
        }
    }
}

impl<D: ReadState<T=Vector3<f32>>, C: ReadState<T=Color>, I: ReadState<T=f32>> DirectionalLight<D, C, I> {
    pub fn color<C2: IntoReadState<Color>>(self, color: C2) -> DirectionalLight<D, C2::Output, I> {
        DirectionalLight {
            id: self.id,
            color: color.into_read_state(),
            resolution: self.resolution,
            intensity: self.intensity,
            direction: self.direction,
            distance: self.distance,
        }
    }

    pub fn intensity<I2: IntoReadState<f32>>(self, intensity: I2) -> DirectionalLight<D, C, I2::Output> {
        DirectionalLight {
            id: self.id,
            color: self.color,
            resolution: self.resolution,
            intensity: intensity.into_read_state(),
            direction: self.direction,
            distance: self.distance,
        }
    }
}

impl<D: ReadState<T=Vector3<f32>>, C: ReadState<T=Color>, I: ReadState<T=f32>> StateSync for DirectionalLight<D, C, I> {
    fn sync(&mut self, env: &mut EnvironmentStack) -> bool {
        self.direction.sync(env) | self.color.sync(env) | self.intensity.sync(env)
    }
}

impl<D: ReadState<T=Vector3<f32>>, C: ReadState<T=Color>, I: ReadState<T=f32>> Render3d for DirectionalLight<D, C, I> {
    fn render(&mut self, context: &mut RenderContext3d) {
        self.sync(context.env_stack);
        //let new_direction = (Matrix4::from(Euler::new(Deg(0.0), Deg(0.0), Deg(1.0))) * Vector4::new(self.direction.x, self.direction.y, self.direction.z, 1.0));

        //self.direction = Vector3::new(new_direction.x, new_direction.y, new_direction.z);
        if !self.direction.value().is_zero() {
            let color = *self.color.value();//.with_opacity(1.0);
            let intensity = *self.intensity.value();
            let direction = self.direction.value().normalize();
            context.directional(color, intensity, direction)
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

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyNode3d)) {}

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {}

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {}
}