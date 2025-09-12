use std::any::{Any, TypeId};
use carbide::animation::AnimationManager;
use carbide::draw::Rect;
use carbide::event::{KeyboardEvent, KeyboardEventContext, KeyboardEventHandler};
use carbide::lifecycle::{Update, UpdateContext};
use carbide::math::Matrix4;
use carbide::render::{LayerId, RenderContext};
use carbide_core::draw::{Dimension, Position};
use carbide_core::render::Render;
use carbide_core::widget::{CommonWidget, Widget, WidgetExt, WidgetId};
use carbide_core::CommonWidgetImpl;

use crate::camera::{Camera, OrbitCamera};
use crate::node3d_sequence::Node3dSequence;
use crate::render::{ContextFactory3d, InnerRenderContext3d, NoopRenderContext3d, RenderContext3d};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Update, KeyboardEvent)]
pub struct Scene3d<N, C> where N: Node3dSequence, C: Camera {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    target: LayerId,
    render_context: Box<dyn InnerRenderContext3d>,
    render_context_factory_id: TypeId,

    nodes: N,

    camera: C,
}

impl Scene3d<(), OrbitCamera> {
    pub fn new(nodes: impl Node3dSequence, camera: impl Camera) -> Scene3d<impl Node3dSequence, impl Camera> {
        Scene3d {
            id: WidgetId::new(),
            position: Position::origin(),
            dimension: Dimension::default(),
            target: LayerId::new(),
            render_context: Box::new(NoopRenderContext3d),
            render_context_factory_id: TypeId::of::<()>(),
            nodes,
            camera,
        }
    }
}

impl<C: Node3dSequence, V: Camera> CommonWidget for Scene3d<C, V> {
    CommonWidgetImpl!(self, child: (), position: self.position, dimension: self.dimension);
}

impl<C: Node3dSequence, V: Camera> Update for Scene3d<C, V> {
    fn update(&mut self, ctx: &mut UpdateContext) {
        self.camera.update(ctx);
    }
}

impl<C: Node3dSequence, V: Camera> KeyboardEventHandler for Scene3d<C, V> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        self.camera.handle_keyboard_event(event, ctx);
    }
}

impl<C: Node3dSequence, V: Camera> Render for Scene3d<C, V> {
    fn render(&mut self, ctx: &mut RenderContext) {
        ctx.env.extract::<ContextFactory3d>(|factory, env| {
            if factory.type_id() != self.render_context_factory_id {
                self.render_context = (factory.render_context)(env);
                self.render_context_factory_id = factory.type_id();

                println!("Set new render context: {:#?}", self.render_context);
            }
        });

        ctx.layer(self.target, Rect::new(self.position, self.dimension), |layer, env| {
            self.render_context.start();

            self.nodes.foreach_mut(&mut |node| {
                node.render(&mut RenderContext3d {
                    render: &mut *self.render_context,
                    //image: &mut *self.image_context,
                    env,
                });
            });

            let camera_spec = self.camera.to_spec(&layer);

            self.render_context.render(layer, camera_spec, env);
        })
    }
}