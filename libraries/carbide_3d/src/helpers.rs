use carbide::color::rgba_bytes;
use carbide::environment::Environment;
use carbide::math::{Matrix4, SquareMatrix, Vector3};
use carbide::state::{AnyReadState, IntoReadState, ReadState, StateSync};

use crate::material::pbr_material::PbrMaterial;
use crate::material::Material;
use crate::node3d::{AnyNode3d, CommonNode3d, NodeId};
use crate::render::{Render3d, RenderContext3d};
use crate::Mesh;

/// Lines along the axes of the space it is in, from its origin: red along +X, green along +Y and
/// blue along +Z. They are unlit, so they keep their colours whatever the lighting. For finding
/// your way around a scene; place it with `.position()` and the like to show a node's axes.
///
/// The lines are thin boxes, so they keep their thickness in perspective.
#[derive(Debug, Clone)]
pub struct AxisHelper {
    id: NodeId,
    length: f32,
    thickness: f32,
    shown: Box<dyn AnyReadState<T=bool>>,
    /// A unit cube, scaled into each line; one upload for all three.
    line: Mesh,
    materials: [Material; 3],
}

/// The axis colours: X, Y and Z.
const AXIS_COLORS: [(u8, u8, u8); 3] = [(230, 60, 60), (70, 200, 70), (70, 120, 240)];

impl AxisHelper {
    /// Axes `length` long, 1% of that thick, shown.
    pub fn new(length: f32) -> AxisHelper {
        let material = |(r, g, b): (u8, u8, u8)| -> Material {
            PbrMaterial::new().color(rgba_bytes(r, g, b, 1.0)).unlit().into()
        };

        AxisHelper {
            id: NodeId::new(),
            length,
            thickness: length * 0.01,
            shown: Box::new(true),
            line: Mesh::cube(1.0),
            materials: AXIS_COLORS.map(material),
        }
    }

    /// The lines' thickness.
    pub fn thickness(mut self, thickness: f32) -> AxisHelper {
        self.thickness = thickness;
        self
    }

    /// Whether the axes are drawn.
    pub fn shown<S: IntoReadState<bool>>(mut self, shown: S) -> AxisHelper {
        self.shown = Box::new(shown.into_read_state());
        self
    }

    /// The transform that turns the unit cube into the line along `axis` (0, 1 or 2): from the
    /// origin to `length` along the axis, `thickness` across.
    fn line_transform(&self, axis: usize) -> Matrix4<f32> {
        let mut size = Vector3::new(self.thickness, self.thickness, self.thickness);
        let mut center = Vector3::new(0.0, 0.0, 0.0);
        size[axis] = self.length;
        center[axis] = self.length / 2.0;

        Matrix4::from_translation(center) * Matrix4::from_nonuniform_scale(size.x, size.y, size.z)
    }
}

impl StateSync for AxisHelper {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.shown.sync(env)
    }
}

impl Render3d for AxisHelper {
    fn render(&mut self, context: &mut RenderContext3d) {
        self.sync(context.env);

        if !*self.shown.value() {
            return;
        }

        for axis in 0..3 {
            let transform = self.line_transform(axis);
            let line = &self.line;
            context.material(&self.materials[axis], |context| {
                context.transform(&transform, |context| context.mesh(line))
            });
        }
    }
}

impl CommonNode3d for AxisHelper {
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
