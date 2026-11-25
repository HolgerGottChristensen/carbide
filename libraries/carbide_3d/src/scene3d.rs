use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use carbide::draw::Rect;
use carbide::environment::Environment;
use carbide::event::{KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler};
use carbide::lifecycle::{Update, UpdateContext};
use carbide::render::{LayerId, RenderContext};
use carbide::color::{Color, TRANSPARENT, WHITE};
use carbide::state::{AnyReadState, IntoReadState, ReadState, StateSync};
use carbide_core::draw::{Dimension, Position};
use carbide_core::render::Render;
use carbide_core::widget::{CommonWidget, Widget, WidgetId};
use carbide_core::CommonWidgetImpl;

use crate::camera::{Camera, CameraController};
use crate::node3d_sequence::Node3dSequence;
use crate::render::{ContextFactory3d, InnerRenderContext3d, NoopRenderContext3d, RenderContext3d, RenderContext3dId, Tonemapping, ViewSpec};

/// A view of a 3D scene: a widget that renders its nodes through its camera into a layer.
///
/// The nodes are shared by every clone of a `Scene3d`, so a clone is another view of the same
/// scene. Each view has its own camera, camera controller, layer and render context:
///
/// ```ignore
/// let scene = Scene3d::new((cube, light)).camera(Camera::perspective().look_at(eye, target));
///
/// HStack::new((
///     scene.clone().controller(OrbitController::new()),
///     scene.camera(Camera::orthographic(2.0).look_at(above, target)),
/// ))
/// ```
///
/// The view renders with the 3D renderer in the environment (see
/// [`ContextFactory3d`]), and switches renderer when that changes. Without one, it renders
/// nothing.
#[derive(Debug, Widget)]
#[carbide_exclude(Render, Update, KeyboardEvent, MouseEvent)]
pub struct Scene3d<N> where N: Node3dSequence {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    /// Shared by every clone of this view.
    content: Rc<RefCell<Scene3dContent<N>>>,

    camera: Camera,
    controller: Option<Box<dyn CameraController>>,

    /// How this view looks at the scene.
    background: Box<dyn AnyReadState<T=Color>>,
    exposure: Box<dyn AnyReadState<T=f32>>,
    tonemapping: Box<dyn AnyReadState<T=Tonemapping>>,

    layer: LayerId,
    render_context: Box<dyn InnerRenderContext3d>,
    /// The id of the factory that created `render_context`, or `None` while it is the no-op
    /// context the view starts with.
    render_context_id: Option<RenderContext3dId>,
}

/// What the views of one scene share: its nodes, and its ambient light.
#[derive(Debug)]
struct Scene3dContent<N> where N: Node3dSequence {
    nodes: N,
    sky: Box<dyn AnyReadState<T=Color>>,
    ground: Box<dyn AnyReadState<T=Color>>,
    ambient_intensity: Box<dyn AnyReadState<T=f32>>,
}

impl<N: Node3dSequence> Scene3d<N> {
    /// A view of `nodes`, through a perspective camera at the origin looking along +Z.
    pub fn new(nodes: N) -> Scene3d<N> {
        Scene3d {
            id: WidgetId::new(),
            position: Position::origin(),
            dimension: Dimension::default(),
            content: Rc::new(RefCell::new(Scene3dContent {
                nodes,
                sky: Box::new(WHITE),
                ground: Box::new(WHITE),
                ambient_intensity: Box::new(0.1f32),
            })),
            camera: Camera::default(),
            controller: None,
            background: Box::new(TRANSPARENT),
            exposure: Box::new(1.0f32),
            tonemapping: Box::new(Tonemapping::default()),
            layer: LayerId::new(),
            render_context: Box::new(NoopRenderContext3d),
            render_context_id: None,
        }
    }

    /// Show the scene through `camera`.
    pub fn camera(mut self, camera: Camera) -> Scene3d<N> {
        self.camera = camera;
        self
    }

    /// Move the camera with `controller`.
    pub fn controller<C: CameraController>(mut self, controller: C) -> Scene3d<N> {
        self.controller = Some(Box::new(controller));
        self
    }

    /// Draw `background` behind everything in this view. Transparent by default, so the UI behind
    /// the view shows through. It can be an `EnvironmentColor`, following the theme.
    pub fn background<C: IntoReadState<Color>>(mut self, background: C) -> Scene3d<N> {
        self.background = Box::new(background.into_read_state());
        self
    }

    /// Multiply this view's lit colours by `exposure` before tone mapping. 1 by default.
    pub fn exposure<E: IntoReadState<f32>>(mut self, exposure: E) -> Scene3d<N> {
        self.exposure = Box::new(exposure.into_read_state());
        self
    }

    /// Map this view's lit colours to the screen's with `tonemapping`. Khronos PBR Neutral by
    /// default.
    pub fn tonemapping<T: IntoReadState<Tonemapping>>(mut self, tonemapping: T) -> Scene3d<N> {
        self.tonemapping = Box::new(tonemapping.into_read_state());
        self
    }

    /// Light the scene from all around with `sky` from above and `ground` from below, blended by
    /// each surface's direction, both times `intensity`. This is the scene's, not the view's: it
    /// changes every view of the scene. White at 0.1 by default.
    pub fn hemisphere<S: IntoReadState<Color>, G: IntoReadState<Color>, I: IntoReadState<f32>>(self, sky: S, ground: G, intensity: I) -> Scene3d<N> {
        {
            let mut content = self.content.borrow_mut();
            content.sky = Box::new(sky.into_read_state());
            content.ground = Box::new(ground.into_read_state());
            content.ambient_intensity = Box::new(intensity.into_read_state());
        }
        self
    }

    /// Light the scene evenly from all around with `color` times `intensity`: a hemisphere with
    /// the same colour above and below. Changes every view of the scene.
    pub fn ambient<C: IntoReadState<Color>, I: IntoReadState<f32>>(self, color: C, intensity: I) -> Scene3d<N> {
        let color = color.into_read_state();
        self.hemisphere(color.clone(), color, intensity)
    }

    /// Recreate the render context if the factory in the environment is not the one that created
    /// it. Keeps the current context if the environment has no factory, so a view without a 3D
    /// renderer keeps rendering nothing. Returns whether the context was recreated.
    fn recreate_render_context_if_factory_changed(&mut self, env: &mut Environment) -> bool {
        let Some(&factory) = env.get::<ContextFactory3d>() else {
            return false;
        };

        if Some(factory.id) == self.render_context_id {
            return false;
        }

        // Drop the old context before creating the new one, so the two never hold their
        // resources at the same time.
        self.render_context = Box::new(NoopRenderContext3d);
        self.render_context = (factory.render_context)(env);
        self.render_context_id = Some(factory.id);

        // A new layer as well: the old one still holds the last frame the old renderer drew, which
        // a renderer that draws nothing would leave showing.
        self.layer = LayerId::new();

        true
    }
}

/// A clone is another view of the same scene. It shares the nodes, and gets a copy of the camera
/// (see [`Camera`]'s `Clone`) and of the controller. It gets a new widget id, a new layer and a
/// new render context: two views rendering into one layer would both show whichever rendered
/// last.
impl<N: Node3dSequence> Clone for Scene3d<N> {
    fn clone(&self) -> Self {
        Scene3d {
            id: WidgetId::new(),
            position: self.position,
            dimension: self.dimension,
            content: self.content.clone(),
            camera: self.camera.clone(),
            controller: self.controller.clone(),
            background: self.background.clone(),
            exposure: self.exposure.clone(),
            tonemapping: self.tonemapping.clone(),
            layer: LayerId::new(),
            render_context: Box::new(NoopRenderContext3d),
            render_context_id: None,
        }
    }
}

impl<N: Node3dSequence> CommonWidget for Scene3d<N> {
    CommonWidgetImpl!(self, child: (), position: self.position, dimension: self.dimension);
}

impl<N: Node3dSequence> Update for Scene3d<N> {
    fn update(&mut self, ctx: &mut UpdateContext) {
        if let Some(controller) = &mut self.controller {
            controller.update(&mut self.camera, ctx);
        }
    }
}

impl<N: Node3dSequence> KeyboardEventHandler for Scene3d<N> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        if let Some(controller) = &mut self.controller {
            controller.handle_keyboard_event(event, &mut self.camera, ctx);
        }
    }
}

impl<N: Node3dSequence> MouseEventHandler for Scene3d<N> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        if let Some(controller) = &mut self.controller {
            controller.handle_mouse_event(event, Rect::new(self.position, self.dimension), &mut self.camera, ctx);
        }
    }
}

impl<N: Node3dSequence> Render for Scene3d<N> {
    fn render(&mut self, ctx: &mut RenderContext) {
        self.recreate_render_context_if_factory_changed(ctx.env);
        self.camera.sync(ctx.env);
        self.background.sync(ctx.env);
        self.exposure.sync(ctx.env);
        self.tonemapping.sync(ctx.env);

        let view = ViewSpec {
            background: *self.background.value(),
            exposure: *self.exposure.value(),
            tonemapping: *self.tonemapping.value(),
        };

        ctx.layer(self.layer, Rect::new(self.position, self.dimension), |layer, env| {
            self.render_context.start();

            // Borrowed only while the nodes render. Nodes must not reach back into their own
            // scene from `render`.
            let mut content = self.content.borrow_mut();

            content.sky.sync(env);
            content.ground.sync(env);
            content.ambient_intensity.sync(env);
            self.render_context.hemisphere(*content.sky.value(), *content.ground.value(), *content.ambient_intensity.value());

            content.nodes.foreach_mut(&mut |node| {
                node.render(&mut RenderContext3d {
                    render: &mut *self.render_context,
                    env,
                });
            });

            let (width, height) = layer.dimensions();
            let aspect_ratio = width as f32 / height.max(1) as f32;

            self.render_context.render(layer, self.camera.spec(aspect_ratio), view, env);
        })
    }
}
