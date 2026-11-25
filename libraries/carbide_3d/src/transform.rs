use carbide::environment::Environment;
use carbide::impl_state_value;
use carbide::math::{Deg, Euler, InnerSpace, Matrix4, One, Quaternion, Rad, Rotation3, Vector3};
use carbide::state::{AnyReadState, ConvertIntoRead, IntoReadState, Map1, RMap1, ReadState, StateSync};

use crate::node3d::{AnyNode3d, CommonNode3d, NodeId};
use crate::render::{Render3d, RenderContext3d};

/// A rotation, as given to [`Transformed::rotation`]. Stored as a quaternion.
///
/// A state of a `Quaternion<f32>`, an `Euler<Deg<f32>>` or an `Euler<Rad<f32>>` converts into it,
/// so a rotation can be given in whichever form suits, for example Euler angles from a slider.
/// For an axis and an angle, use [`Rotation::axis_angle`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rotation(pub Quaternion<f32>);

impl Rotation {
    /// No rotation.
    pub fn identity() -> Rotation {
        Rotation(Quaternion::one())
    }

    /// A rotation of `angle` around `axis`. The axis doesn't need to be normalised.
    pub fn axis_angle<A: Into<Rad<f32>>>(axis: Vector3<f32>, angle: A) -> Rotation {
        Rotation(Quaternion::from_axis_angle(axis.normalize(), angle))
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        Matrix4::from(self.0)
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Rotation::identity()
    }
}

impl_state_value!(Rotation);

impl ConvertIntoRead<Rotation> for Quaternion<f32> {
    type Output<G: AnyReadState<T=Quaternion<f32>> + Clone> = RMap1<fn(&Quaternion<f32>) -> Rotation, Quaternion<f32>, Rotation, G>;

    fn convert<F: AnyReadState<T=Quaternion<f32>> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |quaternion| Rotation(*quaternion))
    }
}

impl ConvertIntoRead<Rotation> for Euler<Deg<f32>> {
    type Output<G: AnyReadState<T=Euler<Deg<f32>>> + Clone> = RMap1<fn(&Euler<Deg<f32>>) -> Rotation, Euler<Deg<f32>>, Rotation, G>;

    fn convert<F: AnyReadState<T=Euler<Deg<f32>>> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |euler| Rotation(Quaternion::from(*euler)))
    }
}

impl ConvertIntoRead<Rotation> for Euler<Rad<f32>> {
    type Output<G: AnyReadState<T=Euler<Rad<f32>>> + Clone> = RMap1<fn(&Euler<Rad<f32>>) -> Rotation, Euler<Rad<f32>>, Rotation, G>;

    fn convert<F: AnyReadState<T=Euler<Rad<f32>>> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |euler| Rotation(Quaternion::from(*euler)))
    }
}

/// A scale, as given to [`Transformed::scale`]: a factor per axis. A state of an `f32` converts
/// into a uniform scale, and a state of a `Vector3<f32>` into a scale per axis.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Scale(pub Vector3<f32>);

impl Scale {
    /// No scaling.
    pub fn identity() -> Scale {
        Scale(Vector3::new(1.0, 1.0, 1.0))
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        Matrix4::from_nonuniform_scale(self.0.x, self.0.y, self.0.z)
    }
}

impl Default for Scale {
    fn default() -> Self {
        Scale::identity()
    }
}

impl_state_value!(Scale);

impl ConvertIntoRead<Scale> for f32 {
    type Output<G: AnyReadState<T=f32> + Clone> = RMap1<fn(&f32) -> Scale, f32, Scale, G>;

    fn convert<F: AnyReadState<T=f32> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |factor| Scale(Vector3::new(*factor, *factor, *factor)))
    }
}

impl ConvertIntoRead<Scale> for Vector3<f32> {
    type Output<G: AnyReadState<T=Vector3<f32>> + Clone> = RMap1<fn(&Vector3<f32>) -> Scale, Vector3<f32>, Scale, G>;

    fn convert<F: AnyReadState<T=Vector3<f32>> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |factors| Scale(*factors))
    }
}

/// A node moved, turned and scaled: the modifier that `.position()`, `.rotation()`, `.scale()`,
/// `.pivot()` and `.transform()` wrap any node in.
///
/// The parts are states, so each can be animated on its own. They are composed in a fixed order,
/// whatever order they are set in: the node is scaled and rotated around the pivot, then moved by
/// the position. As a matrix, `translation · pivot · rotation · scale · pivot⁻¹`. A whole matrix
/// given with [`transform`](Transformed::transform) replaces the composed parts.
///
/// Calling one of these methods on a `Transformed` sets that part of it, instead of wrapping it
/// in another one.
#[derive(Debug, Clone)]
pub struct Transformed<N> where N: AnyNode3d + Clone {
    id: NodeId,
    child: N,
    position: Box<dyn AnyReadState<T=Vector3<f32>>>,
    rotation: Box<dyn AnyReadState<T=Rotation>>,
    scale: Box<dyn AnyReadState<T=Scale>>,
    pivot: Box<dyn AnyReadState<T=Vector3<f32>>>,
    matrix: Option<Box<dyn AnyReadState<T=Matrix4<f32>>>>,
}

impl<N: AnyNode3d + Clone> Transformed<N> {
    /// `child`, not yet moved, turned or scaled.
    pub fn new(child: N) -> Transformed<N> {
        Transformed {
            id: NodeId::new(),
            child,
            position: Box::new(Vector3::new(0.0, 0.0, 0.0)),
            rotation: Box::new(Rotation::identity()),
            scale: Box::new(Scale::identity()),
            pivot: Box::new(Vector3::new(0.0, 0.0, 0.0)),
            matrix: None,
        }
    }

    /// Move the node by `position`.
    pub fn position<P: IntoReadState<Vector3<f32>>>(mut self, position: P) -> Transformed<N> {
        self.position = Box::new(position.into_read_state());
        self
    }

    /// Turn the node by `rotation`, around its pivot: a state of a `Rotation`, a `Quaternion<f32>`,
    /// or Euler angles in degrees or radians.
    pub fn rotation<R: IntoReadState<Rotation>>(mut self, rotation: R) -> Transformed<N> {
        self.rotation = Box::new(rotation.into_read_state());
        self
    }

    /// Scale the node around its pivot: a state of a uniform `f32` factor, a `Vector3<f32>` of
    /// factors per axis, or a `Scale`.
    pub fn scale<S: IntoReadState<Scale>>(mut self, scale: S) -> Transformed<N> {
        self.scale = Box::new(scale.into_read_state());
        self
    }

    /// The point, in the node's own space, that it is rotated and scaled around. The origin by
    /// default.
    pub fn pivot<P: IntoReadState<Vector3<f32>>>(mut self, pivot: P) -> Transformed<N> {
        self.pivot = Box::new(pivot.into_read_state());
        self
    }

    /// Use `matrix` as the whole transform, instead of composing the position, rotation, scale
    /// and pivot.
    pub fn transform<M: IntoReadState<Matrix4<f32>>>(mut self, matrix: M) -> Transformed<N> {
        self.matrix = Some(Box::new(matrix.into_read_state()));
        self
    }

    /// The transform from the node's space to its parent's, as of the last sync.
    pub fn matrix(&self) -> Matrix4<f32> {
        if let Some(matrix) = &self.matrix {
            return *matrix.value();
        }

        let pivot = *self.pivot.value();

        Matrix4::from_translation(*self.position.value())
            * Matrix4::from_translation(pivot)
            * self.rotation.value().matrix()
            * self.scale.value().matrix()
            * Matrix4::from_translation(-pivot)
    }
}

impl<N: AnyNode3d + Clone> StateSync for Transformed<N> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        let matrix = match &mut self.matrix {
            Some(matrix) => matrix.sync(env),
            None => false,
        };

        self.position.sync(env) | self.rotation.sync(env) | self.scale.sync(env) | self.pivot.sync(env) | matrix
    }
}

impl<N: AnyNode3d + Clone> Render3d for Transformed<N> {
    fn render(&mut self, context: &mut RenderContext3d) {
        self.sync(context.env);
        let matrix = self.matrix();

        let child = &mut self.child;
        context.transform(&matrix, |context| child.render(context));
    }
}

impl<N: AnyNode3d + Clone> CommonNode3d for Transformed<N> {
    fn id(&self) -> NodeId {
        self.id
    }

    fn transform(&self) -> Matrix4<f32> {
        self.matrix()
    }

    fn visible(&self) -> bool {
        true
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyNode3d)) {
        f(&self.child)
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {
        f(&mut self.child)
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {
        f(&mut self.child)
    }
}
