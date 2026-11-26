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
use crate::camera::controller::contains;
use crate::node3d::{AnyNode3d, NodeId};
use crate::picking::{PickHit, PickRecorder, PickTolerance, PointerEvent3d};
use carbide::event::MouseButton;
use carbide_core::animation::AnimationManager;
use carbide_core::scene::SceneManager;
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
    /// Whether the view responds to the pointer and keyboard (see `interactive`).
    interactive: Box<dyn AnyReadState<T=bool>>,
    exposure: Box<dyn AnyReadState<T=f32>>,
    tonemapping: Box<dyn AnyReadState<T=Tonemapping>>,

    layer: LayerId,
    render_context: Box<dyn InnerRenderContext3d>,
    /// The id of the factory that created `render_context`, or `None` while it is the no-op
    /// context the view starts with.
    render_context_id: Option<RenderContext3dId>,

    /// What the last frame recorded for picking: the path of nodes behind each pick id.
    picking: PickRecorder,
    /// Whether the app has called [`pick`](Scene3d::pick): the picking buffer is then rendered
    /// with every frame, as it is when a node has a pointer handler.
    pick_requested: bool,
    /// The path under the pointer, for hover enter and leave.
    hovered: Vec<NodeId>,
    /// Where the left button was pressed inside the view, for clicks: on a node, or on nothing
    /// pickable (`Some(None)`, for missed clicks).
    pressed: Option<Option<NodeId>>,
    /// Called for a click inside the view that hits nothing pickable (see `on_missed_click`).
    on_missed_click: Option<MissedClick>,
    /// Where the pointer is, while it is inside the view: hover is picked again there each
    /// update, so it follows the scene moving under a still pointer.
    pointer: Option<Position>,
    /// Whether the frame rendering now is the extra frame asked for to check hover against the
    /// frame before it.
    hover_check: bool,
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
    /// A view of `nodes`, through a perspective camera at the origin looking along -Z.
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
            interactive: Box::new(true),
            exposure: Box::new(1.0f32),
            tonemapping: Box::new(Tonemapping::default()),
            layer: LayerId::new(),
            render_context: Box::new(NoopRenderContext3d),
            render_context_id: None,
            picking: PickRecorder::default(),
            pick_requested: false,
            hovered: vec![],
            pressed: None,
            on_missed_click: None,
            pointer: None,
            hover_check: false,
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

    /// Whether the view responds to the pointer and the keyboard, a value or a state of one:
    /// `true` by default, as for charts. While it is `false`, the view ignores them: nothing is
    /// picked, hovered or clicked, and its camera controller doesn't orbit, zoom or turn it. Turning
    /// it off lets go of the pointer, so hovered nodes hear the pointer leave and `.hovered(..)`
    /// states are set to `false`. The camera can still be moved from code, through its state.
    pub fn interactive<I: IntoReadState<bool>>(mut self, interactive: I) -> Scene3d<N> {
        self.interactive = Box::new(interactive.into_read_state());
        self
    }

    /// Call `f` when a click inside the view hits nothing pickable: both the press and the release
    /// are on the background, or on meshes that aren't pickable, without a drag in between. As
    /// react-three-fiber's `onPointerMissed`; for example, to clear a selection. Clones of the view
    /// share `f`.
    pub fn on_missed_click<F: FnMut(&mut Environment) + 'static>(mut self, f: F) -> Scene3d<N> {
        self.on_missed_click = Some(MissedClick(Rc::new(RefCell::new(f))));
        self
    }

    /// Whether anything in the view listens to the pointer: pointer handlers on nodes, or a
    /// missed-click handler.
    fn listening(&self) -> bool {
        self.picking.listeners > 0 || self.on_missed_click.is_some()
    }

    /// Whether the view responds to the pointer and keyboard, after syncing the state.
    fn is_interactive(&mut self, env: &mut Environment) -> bool {
        self.interactive.sync(env);
        *self.interactive.value()
    }

    /// Let go of the pointer, when the view stops responding to it: forget where it is and what
    /// was pressed, and end hover.
    fn release_pointer(&mut self, env: &mut Environment) {
        self.pointer = None;
        self.pressed = None;
        if !self.hovered.is_empty() {
            self.set_hovered(None, env);
        }
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
            interactive: self.interactive.clone(),
            exposure: self.exposure.clone(),
            tonemapping: self.tonemapping.clone(),
            layer: LayerId::new(),
            render_context: Box::new(NoopRenderContext3d),
            render_context_id: None,
            picking: PickRecorder::default(),
            pick_requested: false,
            hovered: vec![],
            pressed: None,
            on_missed_click: self.on_missed_click.clone(),
            pointer: None,
            hover_check: false,
        }
    }
}

impl<N: Node3dSequence> CommonWidget for Scene3d<N> {
    CommonWidgetImpl!(self, child: (), position: self.position, dimension: self.dimension);
}

impl<N: Node3dSequence> Update for Scene3d<N> {
    fn update(&mut self, ctx: &mut UpdateContext) {
        if !self.is_interactive(ctx.env) {
            self.release_pointer(ctx.env);
        }

        // Hover under a still pointer: pick again against the last frame, which the GPU has
        // usually finished by now. If hover changed, a handler may have changed the scene, so
        // look again next frame.
        if let Some(pointer) = self.pointer {
            if self.picking.hover_listeners > 0 {
                let hit = self.pick_at(pointer, ctx.env);
                if self.set_hovered(hit, ctx.env) {
                    request_frame(ctx.env);
                }
            }
        }

        if let Some(controller) = &mut self.controller {
            controller.update(&mut self.camera, ctx);
        }
    }
}

/// Ask for another frame, as animations do.
fn request_frame(env: &mut Environment) {
    if let Some(manager) = env.get_mut::<AnimationManager>() {
        manager.request_animation_frame();
    }
}

impl<N: Node3dSequence> KeyboardEventHandler for Scene3d<N> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        if !self.is_interactive(ctx.env) {
            return;
        }

        if let Some(controller) = &mut self.controller {
            controller.handle_keyboard_event(event, &mut self.camera, ctx);
        }
    }
}

impl<N: Node3dSequence> Scene3d<N> {
    /// The node under `position`, a position in the window as mouse events give them, as the
    /// last rendered frame shows it. `None` outside the view, where nothing pickable is, or with a
    /// renderer that can't pick. Calling it makes the view render a picking buffer with each
    /// frame from then on, so the first call may find nothing. See [`picking`](crate::picking).
    pub fn pick(&mut self, position: Position, env: &mut Environment) -> Option<PickHit> {
        self.pick_requested = true;
        self.pick_at(position, env)
    }

    fn pick_at(&mut self, position: Position, env: &mut Environment) -> Option<PickHit> {
        if !contains(Rect::new(self.position, self.dimension), position) {
            return None;
        }

        // Physical pixels in the layer, which starts at the view's position: layers are the
        // view's logical size times the scale factor.
        let scale = env.get_mut::<SceneManager>().map(|manager| manager.scale_factor()).unwrap_or(1.0);
        let x = ((position.x - self.position.x) * scale).floor().max(0.0) as u32;
        let y = ((position.y - self.position.y) * scale).floor().max(0.0) as u32;
        let tolerance = env.get::<PickTolerance>().copied().unwrap_or(2.0).max(0.0) as f64;
        let radius = ((tolerance * scale).round() as u32).min(7);

        let sample = self.render_context.pick((x, y), radius, env)?;
        let path = self.picking.path(sample.pick_id)?.clone();

        Some(PickHit {
            node: *path.last()?,
            path,
            position: sample.position,
            normal: sample.normal,
            distance: sample.distance,
        })
    }

    /// Deliver `event` to the node `id`, wherever it is in the scene.
    fn deliver(&mut self, id: NodeId, event: &PointerEvent3d, env: &mut Environment) {
        fn visit(node: &mut dyn AnyNode3d, id: NodeId, event: &PointerEvent3d, env: &mut Environment, done: &mut bool) {
            if *done {
                return;
            }
            if node.id() == id {
                node.process_pointer_event(event, env);
                *done = true;
                return;
            }
            node.foreach_child_mut(&mut |child| visit(child, id, event, env, done));
        }

        let mut done = false;
        let mut content = self.content.borrow_mut();
        content.nodes.foreach_mut(&mut |node| visit(node, id, event, env, &mut done));
    }

    /// Hover enter and leave: the nodes on the new path and not the old hear `Enter`, outermost
    /// first; those on the old path and not the new hear `Leave`, innermost first.
    /// Returns whether the hovered path changed.
    fn set_hovered(&mut self, hit: Option<PickHit>, env: &mut Environment) -> bool {
        let new_path = hit.as_ref().map(|hit| hit.path.clone()).unwrap_or_default();
        let old_path = std::mem::replace(&mut self.hovered, new_path.clone());
        let changed = old_path != new_path;

        for id in old_path.iter().rev().filter(|id| !new_path.contains(id)) {
            self.deliver(*id, &PointerEvent3d::Leave, env);
        }

        if let Some(hit) = hit {
            for id in new_path.iter().filter(|id| !old_path.contains(id)) {
                self.deliver(*id, &PointerEvent3d::Enter(hit.clone()), env);
            }
        }

        changed
    }

    /// Hover and clicks for the nodes with pointer handlers. Only while something listens.
    fn handle_pointer(&mut self, event: &MouseEvent, env: &mut Environment) {
        if !self.listening() {
            return;
        }

        match event {
            MouseEvent::Move { to, .. } => {
                let inside = contains(Rect::new(self.position, self.dimension), *to);
                if inside {
                    // Hover is picked at most once a frame, by the next update, not on every move:
                    // each pick waits for the GPU. Ask for that frame, if anything listens for
                    // hover. On entering, ask for one anyway: it renders the picking buffer the
                    // view doesn't have yet, which clicks read.
                    let entering = self.pointer.is_none();
                    if entering || (self.picking.hover_listeners > 0 && self.pointer != Some(*to)) {
                        request_frame(env);
                    }
                    self.pointer = Some(*to);
                } else {
                    // Leaving needs no pick.
                    self.pointer = None;
                    if !self.hovered.is_empty() {
                        self.set_hovered(None, env);
                    }
                }
            }
            MouseEvent::Press { button: MouseButton::Left, position, .. } => {
                self.pressed = if contains(Rect::new(self.position, self.dimension), *position) {
                    Some(self.pick_at(*position, env).map(|hit| hit.node))
                } else {
                    None
                };
            }
            // A quick second click at the same place arrives as an `NClick`; each is a click.
            MouseEvent::Click(MouseButton::Left, position, _) | MouseEvent::NClick(MouseButton::Left, position, _, _) => {
                if !contains(Rect::new(self.position, self.dimension), *position) {
                    return;
                }
                let Some(hit) = self.pick_at(*position, env) else {
                    // Pressed and released on nothing pickable: a missed click.
                    if self.pressed == Some(None) {
                        if let Some(MissedClick(f)) = &self.on_missed_click {
                            (f.borrow_mut())(env);
                        }
                    }
                    return;
                };
                if self.pressed != Some(Some(hit.node)) {
                    return;
                }
                // Bubbling: innermost first.
                for id in hit.path.iter().rev() {
                    self.deliver(*id, &PointerEvent3d::Click(hit.clone()), env);
                }
            }
            _ => {}
        }
    }
}

impl<N: Node3dSequence> MouseEventHandler for Scene3d<N> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        if !self.is_interactive(ctx.env) {
            self.release_pointer(ctx.env);
            return;
        }

        self.handle_pointer(event, ctx.env);

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

        let mut view = ViewSpec {
            background: *self.background.value(),
            exposure: *self.exposure.value(),
            tonemapping: *self.tonemapping.value(),
            picking: false,
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

            self.picking.clear();
            content.nodes.foreach_mut(&mut |node| {
                node.render(&mut RenderContext3d {
                    render: &mut *self.render_context,
                    env,
                    picking: &mut self.picking,
                });
            });


            // A picking buffer only when something can use it: handlers and the pointer inside
            // this view (it is in at most one), or the app picking by itself.
            view.picking = (self.listening() && self.pointer.is_some()) || self.pick_requested;

            let (width, height) = layer.dimensions();
            let aspect_ratio = width as f32 / height.max(1) as f32;

            self.render_context.render(layer, self.camera.spec(aspect_ratio), view, env);
        });

        // With the pointer inside, this frame's picking buffer is checked for hover on the next
        // update. If no other frame would come, ask for one; but not after the frame that was
        // itself only asked for that, or a still scene would never stop.
        if self.pointer.is_some() && self.picking.hover_listeners > 0 && !self.hover_check {
            request_frame(ctx.env);
            self.hover_check = true;
        } else {
            self.hover_check = false;
        }
    }
}

/// A missed-click handler, shared by the clones of a view.
#[derive(Clone)]
struct MissedClick(Rc<RefCell<dyn FnMut(&mut Environment)>>);

impl Debug for MissedClick {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MissedClick")
    }
}
