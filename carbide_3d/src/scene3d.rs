use carbide::draw::Rect;
use carbide::render::{LayerId, RenderContext};
use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::render::Render;
use carbide_core::widget::{CommonWidget, Widget, WidgetExt, WidgetId};

use crate::{InnerRenderContext3d, render_context3d, RenderContext3d};
use crate::camera::SimpleCamera;
use crate::node3d_sequence::Node3dSequence;
use crate::render3d::Render3d;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Scene3d<C> where C: Node3dSequence {
    id: WidgetId,
    position: Position,
    dimension: Dimension,

    target: LayerId,
    context: Box<dyn InnerRenderContext3d>,

    nodes: C,

    camera: SimpleCamera,
}

impl Scene3d<()> {
    pub fn new(nodes: impl Node3dSequence, camera: SimpleCamera) -> Scene3d<impl Node3dSequence> {
        let context = render_context3d();

        Scene3d {
            id: WidgetId::new(),
            position: Position::origin(),
            dimension: Dimension::default(),
            target: LayerId::new(),
            context,
            nodes,
            camera,
        }
    }
}

impl<C: Node3dSequence> CommonWidget for Scene3d<C> {
    CommonWidgetImpl!(self, id: self.id, child: (), position: self.position, dimension: self.dimension);
}

impl<C: Node3dSequence> Render for Scene3d<C> {
    fn render(&mut self, context: &mut RenderContext) {
        context.env.request_animation_frame();
        context.layer(self.target, Rect::new(self.position, self.dimension), |layer, env| {
            self.context.start();

            self.nodes.foreach_mut(&mut |node| {
                node.render(&mut RenderContext3d {
                    render: &mut *self.context,
                    env,
                });
            });

            self.context.render(layer, &self.camera);
        })
    }
}

impl<C: Node3dSequence> WidgetExt for Scene3d<C> {}