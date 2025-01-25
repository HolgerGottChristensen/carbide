use carbide::animation::AnimationManager;
use carbide::draw::Rect;
use carbide::render::{LayerId, RenderContext};
use carbide::render::matrix::Matrix4;
use carbide::state::ReadState;
use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::render::Render;
use carbide_core::widget::{CommonWidget, Widget, WidgetExt, WidgetId};

use crate::{image_context3d, InnerImageContext3d, InnerRenderContext3d, render_context3d, RenderContext3d};
use crate::camera::SimpleCamera;
use crate::node3d_sequence::Node3dSequence;
use crate::render3d::Render3d;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Scene3d<C, V> where C: Node3dSequence, V: ReadState<T=Matrix4<f32>> {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    target: LayerId,
    context: Box<dyn InnerRenderContext3d>,
    image_context: Box<dyn InnerImageContext3d>,

    nodes: C,

    camera: SimpleCamera<V>,
}

impl Scene3d<(), Matrix4<f32>> {
    pub fn new<V: ReadState<T=Matrix4<f32>>>(nodes: impl Node3dSequence, camera: SimpleCamera<V>) -> Scene3d<impl Node3dSequence, V> {
        let context = render_context3d();
        let image_context = image_context3d();

        Scene3d {
            id: WidgetId::new(),
            position: Position::origin(),
            dimension: Dimension::default(),
            target: LayerId::new(),
            context,
            image_context,
            nodes,
            camera,
        }
    }
}

impl<C: Node3dSequence, V: ReadState<T=Matrix4<f32>>> CommonWidget for Scene3d<C, V> {
    CommonWidgetImpl!(self, child: (), position: self.position, dimension: self.dimension);
}

impl<C: Node3dSequence, V: ReadState<T=Matrix4<f32>>> Render for Scene3d<C, V> {
    fn render(&mut self, ctx: &mut RenderContext) {
        AnimationManager::get(ctx.env_stack, |manager| {
            manager.request_animation_frame();
        });

        ctx.layer(self.target, Rect::new(self.position, self.dimension), |layer, env_stack| {
            self.context.start();

            self.nodes.foreach_mut(&mut |node| {
                node.render(&mut RenderContext3d {
                    render: &mut *self.context,
                    image: &mut *self.image_context,
                    env_stack,
                });
            });

            self.context.render(layer, &self.camera);
        })
    }
}