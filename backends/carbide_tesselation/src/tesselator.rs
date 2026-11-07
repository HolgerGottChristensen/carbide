use std::fmt::Debug;
use std::rc::Rc;
use carbide_core::draw::{DrawShape, Position};
use carbide_core::draw::fill::FillOptions;
use carbide_core::draw::stroke::StrokeOptions;
use carbide_core::environment::{Environment, EnvironmentKey, Shared};
use crate::stroke_vertex::StrokeVertex;
use crate::triangle::Triangle;

/// A tesselator converts shapes into triangles that can be rendered.
///
/// The trait is dyn compatible, such that a renderer can use any implementation
/// registered into the environment through the [TesselatorKey].
pub trait Tesselator: Debug + 'static {
    /// Tesselate the inside of the shape into a list of triangles.
    ///
    /// The `tolerance_multiplier` scales the precision of the tesselation. It should
    /// be increased when the shape is scaled up, to avoid visible segments on curves.
    /// A value of `1.0` means the default tolerance of the implementation.
    fn fill(&self, shape: &DrawShape, options: &FillOptions, tolerance_multiplier: f32) -> Vec<Triangle<Position>>;

    /// Tesselate the outline of the shape into a list of triangles.
    ///
    /// The `tolerance_multiplier` scales the precision of the tesselation. It should
    /// be increased when the shape is scaled up, to avoid visible segments on curves.
    /// A value of `1.0` means the default tolerance of the implementation.
    fn stroke(&self, shape: &DrawShape, options: &StrokeOptions, tolerance_multiplier: f32) -> Vec<Triangle<StrokeVertex>>;
}

/// The key used to look up the [Tesselator] from the environment.
///
/// The value is reference counted, so renderers can retrieve and keep a
/// handle to the tesselator, independent of the lifetime of the environment.
#[derive(Copy, Clone, Debug)]
pub struct TesselatorKey;

impl EnvironmentKey for TesselatorKey {
    type Value = Rc<dyn Tesselator>;
    type Access = Shared;
}

impl TesselatorKey {
    /// Insert the tesselator into the environment, making it the tesselator retrieved by [TesselatorKey::get].
    pub fn insert(env: &mut Environment<'static>, tesselator: impl Tesselator) {
        let value: Rc<dyn Tesselator> = Rc::new(tesselator);
        env.insert::<TesselatorKey>(Box::leak(Box::new(value)));
    }

    /// Get a handle to the tesselator registered in the environment, if any.
    pub fn get(env: &Environment) -> Option<Rc<dyn Tesselator>> {
        env.get::<TesselatorKey>().cloned()
    }
}
