use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use wgpu::CommandEncoderDescriptor;

use carbide_core::accessibility::AccessibilityContext;
use carbide_core::animation::AnimationManager;
use carbide_core::asynchronous::{check_tasks, set_event_sink, AsyncContext};
use carbide_core::draw::theme::Theme;
use carbide_core::draw::{Alignment, Dimension, ImageId, Position, Rect, Scalar};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::event::{AccessibilityEvent, AccessibilityEventContext, CoreEvent, EventSink, WindowEvent, WindowEventContext};
use carbide_core::focus::FocusManager;
use carbide_core::layout::LayoutContext;
use carbide_core::lifecycle::UpdateContext;
use carbide_core::render::{NoopRenderContext, RenderContext, RenderInstruction, RenderInstructionCache};
use carbide_core::scene::{AnyScene, SceneManager};
use carbide_core::widget::managers::{FontSizeManager, ThemeManager};
use carbide_core::widget::{NavigationStack, Rectangle, Widget, WidgetExt, WidgetId, ZStack};
use carbide_winit::dpi::{PhysicalPosition, PhysicalSize};
use carbide_winit::event::{ElementState, MouseButton, MouseScrollDelta};
use carbide_winit::keyboard::Key;
use carbide_winit::window::WindowId;
use carbide_winit::{update_scale_factor, NewEventHandler};

use carbide_cosmic_text::text_context::CosmicTextContext;
use crate::image_context::WGPUImageContext;
use crate::offscreen_renderer::OffscreenRenderer;
use crate::testing::{snapshot, SnapshotOptions, TestScene};
use crate::wgpu_context::WgpuContext;
use crate::wgpu_msaa::WgpuMsaa;

/// A fresh `WgpuContext` per `Harness`, deliberately *not* shared across instances.
///
/// Sharing one process-wide `WgpuContext` (cloned per `Harness`) was the original design here, for
/// speed — `WgpuContext::new()` requests an adapter/device and compiles every shader module, which
/// is expensive. It was reverted after discovering a real correctness bug: `WgpuContext::
/// atlas_cache_texture` is a *stateful* glyph cache whose contents are only meaningful in
/// combination with the one `CosmicTextContext`'s allocation bookkeeping that populated it. Two
/// `Harness`es sharing the physical texture while each running its own independent
/// `CosmicTextContext` corrupts each other's glyph/icon atlas regions — observed directly as a
/// `Toggle` checkbox icon rendering wrong when a second `Harness` was constructed while a first
/// one (rendering the same icon) was still alive, even though nothing else about the test was
/// non-deterministic. `Device`/`Queue`/shader modules/pipeline layouts *would* be safe to share
/// (genuinely stateless), but `WgpuContext` bundles the atlas cache in with them, so sharing
/// anything less than the whole struct would need restructuring `WgpuContext` itself — out of
/// scope for this fixture. Correctness over speed: construct fresh every time.
fn new_wgpu_context() -> WgpuContext {
    futures::executor::block_on(WgpuContext::new())
}

/// Installed as the global `carbide_core::asynchronous` event sink for a `Harness`'s lifetime, in
/// place of the real `ProxyEventLoop`/winit event loop wakeup a live app uses. `CoreEvent::Async`
/// is the one signal both carbide's "proper" async API (`task!`/`spawn_task`/`check_tasks`) and
/// hand-rolled background-thread patterns (a plain `mpsc::channel` + a manual
/// `get_event_sink().send(CoreEvent::Async)` notify, e.g. the `mandelbrot` example) already funnel
/// through — intercepting it here gives a bounded, no-busy-poll wait for real background work to
/// finish, covering both patterns without needing to reimplement either.
#[derive(Clone)]
struct TestEventSink {
    inner: Arc<(Mutex<bool>, Condvar)>,
}

impl TestEventSink {
    fn new() -> Self {
        TestEventSink {
            inner: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    /// Blocks until an event has been observed since the last call, or `timeout` elapses.
    /// Resets the flag either way, so the next call only reports events that happened after
    /// this one returns.
    fn wait(&self, timeout: Duration) -> bool {
        let (lock, condvar) = &*self.inner;
        let guard = lock.lock().unwrap();
        // The predicate loop only exits when either the flag is set (an event was observed) or
        // the timeout elapsed while it was still unset — so the flag's final value alone tells
        // us which happened, regardless of `WaitTimeoutResult::timed_out()`.
        let (mut guard, _) = condvar.wait_timeout_while(guard, timeout, |observed| !*observed).unwrap();
        let observed = *guard;
        *guard = false;
        observed
    }
}

impl EventSink for TestEventSink {
    fn send(&self, _: CoreEvent) {
        let (lock, condvar) = &*self.inner;
        let mut observed = lock.lock().unwrap();
        *observed = true;
        condvar.notify_all();
    }
}

/// Runs `f`, with `WgpuContext` temporarily pushed into `env`, and returns `f`'s result.
/// `Environment::with_mut` itself always returns `()` — it's designed for side-effecting scopes,
/// not for threading a return value back out — so capturing the result needs this out-parameter
/// (a local `Option` the inner closure assigns into) rather than relying on the call expression's
/// value directly.
fn with_wgpu_context<R>(env: &mut Environment, wgpu_context: &mut WgpuContext, f: impl FnOnce(&mut Environment) -> R) -> R {
    let mut result: Option<R> = None;
    env.with_mut::<WgpuContext>(wgpu_context, |env| {
        result = Some(f(env));
    });
    result.unwrap()
}

/// `dimension` is always in logical units; every GPU-side resource (textures, readback buffers)
/// needs physical pixels, i.e. `dimension * scale_factor` — not `dimension` verbatim. Rounds up
/// (`.ceil()`) so a fractional logical size never gets truncated to a smaller physical texture
/// than the layout actually requested.
fn physical_size(dimension: Dimension, scale_factor: Scalar) -> PhysicalSize<u32> {
    PhysicalSize::new(
        (dimension.width.max(1.0) * scale_factor).ceil() as u32,
        (dimension.height.max(1.0) * scale_factor).ceil() as u32,
    )
}

/// Every `cargo test` integration-test file compiles to its own binary, named
/// `target/debug/deps/{file_stem}-{16-hex-char disambiguation hash}` — e.g. `tests/rendering.rs`
/// becomes `rendering-5a319949d7f31b78`. `std::env::current_exe()` returns *this* binary's path,
/// so stripping that trailing hash recovers the source file's name (`"rendering"`) at runtime,
/// letting `current_test_scope` group every test in one file under one folder without needing the
/// file name spelled out anywhere by hand. Falls back to the un-stripped stem if a binary somehow
/// isn't named in cargo's usual pattern (e.g. a hand-rolled `[[test]] harness = false` binary).
fn current_test_binary_name() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let stem = exe.file_stem()?.to_str()?.to_string();

    match stem.rsplit_once('-') {
        Some((target, hash)) if hash.len() >= 8 && hash.chars().all(|c| c.is_ascii_hexdigit()) => Some(target.to_string()),
        _ => Some(stem),
    }
}

/// Cargo's default test harness runs each `#[test]` fn in its own thread, named after the test's
/// fully-qualified path (e.g. `"rectangle_basic"`, or `"some_mod::rectangle_basic"` for a test
/// inside a nested module). Combined with `current_test_binary_name`, this builds a two-level
/// scope — `{file}/{test}` (`::` within the test path is further replaced with `/`, so a nested
/// test module scopes to a nested directory too) — so every snapshot a `Harness` produces is
/// automatically grouped by source file and then by test, with no naming-convention discipline
/// required from whoever writes the test: two different tests (in the same file or different
/// files) can each call `snapshot("idle")` without colliding.
///
/// Two `Harness`es built within the *same* test (as `toggle_accessible_and_mouse_paths_agree`
/// does, snapshotting both a mouse-driven and an accessibility-driven harness against the same
/// name to assert parity) get the same scope, since they run on the same thread — the collision
/// there is intentional, not one this scoping is meant to prevent.
///
/// Falls back to a single-level scope, or no scoping at all, if either half is unavailable — e.g.
/// a `Harness` constructed outside of `cargo test`'s default harness altogether.
fn current_test_scope() -> Option<String> {
    let binary = current_test_binary_name();
    let test = std::thread::current().name().map(|name| name.replace("::", "/"));

    match (binary, test) {
        (Some(binary), Some(test)) => Some(format!("{binary}/{test}")),
        (Some(binary), None) => Some(binary),
        (None, Some(test)) => Some(test),
        (None, None) => None,
    }
}

/// Hand-built `serde_json::Value` conversion for `snapshot_render_instructions` — see that
/// method's doc comment for why this doesn't just derive `Serialize` on the underlying types.
fn render_passes_to_json(passes: &[crate::render_pass_command::RenderPass]) -> serde_json::Value {
    use crate::render_pass_command::{RenderPass, RenderPassCommand, WGPUBindGroup};

    fn bind_group_to_json(bind_group: &WGPUBindGroup) -> serde_json::Value {
        match bind_group {
            WGPUBindGroup::Image(id) => serde_json::json!({ "type": "Image", "id": format!("{id:?}") }),
            WGPUBindGroup::Target(index) => serde_json::json!({ "type": "Target", "index": index }),
            WGPUBindGroup::Layer(id) => serde_json::json!({ "type": "Layer", "id": format!("{id:?}") }),
        }
    }

    fn command_to_json(command: &RenderPassCommand) -> serde_json::Value {
        match command {
            RenderPassCommand::SetScissor { rect } => serde_json::json!({
                "type": "SetScissor", "rect": format!("{rect:?}"),
            }),
            RenderPassCommand::Draw { vertex_range } => serde_json::json!({
                "type": "Draw", "vertex_range": [vertex_range.start, vertex_range.end],
            }),
            RenderPassCommand::Stencil { vertex_range } => serde_json::json!({
                "type": "Stencil", "vertex_range": [vertex_range.start, vertex_range.end],
            }),
            RenderPassCommand::DeStencil { vertex_range } => serde_json::json!({
                "type": "DeStencil", "vertex_range": [vertex_range.start, vertex_range.end],
            }),
            RenderPassCommand::Uniform { uniform_bind_group_index } => serde_json::json!({
                "type": "Uniform", "uniform_bind_group_index": uniform_bind_group_index,
            }),
            RenderPassCommand::Gradient(gradient) => serde_json::json!({
                "type": "Gradient", "value": format!("{gradient:?}"),
            }),
            RenderPassCommand::StrokeDashing(dashes) => serde_json::json!({
                "type": "StrokeDashing", "value": format!("{dashes:?}"),
            }),
            RenderPassCommand::SetBindGroup { bind_group } => serde_json::json!({
                "type": "SetBindGroup", "bind_group": bind_group_to_json(bind_group),
            }),
            RenderPassCommand::SetMaskBindGroup { bind_group } => serde_json::json!({
                "type": "SetMaskBindGroup", "bind_group": bind_group_to_json(bind_group),
            }),
        }
    }

    let passes: Vec<serde_json::Value> = passes.iter().map(|pass| match pass {
        RenderPass::Normal { commands, target_index } => serde_json::json!({
            "type": "Normal",
            "target_index": target_index,
            "commands": commands.iter().map(command_to_json).collect::<Vec<_>>(),
        }),
        RenderPass::Clear { target_index } => serde_json::json!({
            "type": "Clear", "target_index": target_index,
        }),
        RenderPass::Filter { vertex_range, filter_id, source_id, target_id, mask_id, initial_copy } => serde_json::json!({
            "type": "Filter",
            "vertex_range": [vertex_range.start, vertex_range.end],
            "filter_id": format!("{filter_id:?}"),
            "source_id": source_id,
            "target_id": target_id,
            "mask_id": mask_id,
            "initial_copy": initial_copy,
        }),
    }).collect();

    serde_json::Value::Array(passes)
}

/// Headless fixture for rendering-regression tests: constructs a single root widget, drives it
/// with synthetic input events (no OS window, no live event loop), renders it to an offscreen
/// texture, and lets a test compare the pixel output (and other snapshot strategies) against a
/// committed reference.
pub struct Harness {
    env: Environment<'static>,
    text_context: CosmicTextContext,
    scenes: [Box<dyn AnyScene>; 1],
    event_handler: NewEventHandler,
    focus_manager: FocusManager,
    animation_manager: AnimationManager,
    render_instruction_cache: HashMap<ImageId, Rc<(Dimension, Vec<RenderInstruction>)>>,
    wgpu_context: WgpuContext,
    offscreen: OffscreenRenderer,
    dimension: Dimension,
    scale_factor: Scalar,
    window_id: WindowId,
    failures: Vec<String>,
    virtual_now: Instant,
    async_signal: TestEventSink,
    test_scope: Option<String>,
}

impl Harness {
    /// Equivalent to `new_with_options(root, dimension, 1.0, WgpuMsaa::X1)` — scale factor 1.0
    /// (logical == physical pixels) and no multisampling, the right default for most tests.
    pub fn new<C: Widget>(root: C, dimension: Dimension) -> Self {
        Self::new_with_options(root, dimension, 1.0, WgpuMsaa::X1)
    }

    /// Like `new`, but lets a test exercise HiDPI-scaled layout/rendering (`scale_factor != 1.0`)
    /// or multisampling (`msaa`) explicitly. Both are fixed for the `Harness`'s lifetime — MSAA
    /// especially so, since `WgpuContext::pipelines` is keyed only by `TextureFormat`, not by
    /// `(TextureFormat, WgpuMsaa)`; changing MSAA after the first pipeline for a format has been
    /// created would silently keep using the *first* sample count requested, not the new one.
    /// `resize()` keeps whatever `scale_factor`/`msaa` the `Harness` was constructed with.
    pub fn new_with_options<C: Widget>(root: C, dimension: Dimension, scale_factor: Scalar, msaa: WgpuMsaa) -> Self {
        let window_id = WindowId::dummy();
        update_scale_factor(window_id, scale_factor);

        let mut wgpu_context = new_wgpu_context();
        let physical = physical_size(dimension, scale_factor);

        let mut env: Environment<'static> = Environment::new();
        let offscreen = with_wgpu_context(&mut env, &mut wgpu_context, |env| {
            OffscreenRenderer::new(physical, scale_factor, msaa, env)
        });

        // Mirrors `Window::new`'s wrapping exactly. In order, innermost first:
        // - `controls_overlay` gives any `carbide_controls` widget that renders into an overlay
        //   (dropdown/menu/popover) somewhere to render into — without it, those panic looking up
        //   the `OverlayManager` environment key.
        // - The background `ZStack` fills in `EnvironmentColor::SystemBackground` behind the
        //   root, exactly like a real window's backdrop, so snapshots reflect what a widget
        //   actually looks like composited over a window rather than over nothing (transparent).
        // - `NavigationStack::new_root` provides the `NavigationManager` environment key that
        //   programmatic-navigation widgets look up.
        // - `ThemeManager`/`FontSizeManager` provide default environment values (theme colors,
        //   font sizes) that most non-trivial widgets read unconditionally with no fallback —
        //   confirmed: an `EnvironmentFontSize::Body` lookup with neither present is an
        //   unconditional `.unwrap()` deep in carbide_core.
        #[cfg(feature = "controls")]
        let root = carbide_controls::controls_overlay(root);

        let root = ZStack::new((
            Rectangle::new().fill(EnvironmentColor::SystemBackground),
            root,
        ));

        let root = NavigationStack::new_root(root);

        let root = FontSizeManager::new(ThemeManager::new(root));
        let scene: Box<dyn AnyScene> = Box::new(TestScene::new(root, dimension));

        // Claims the thread-local async event sink for this harness's lifetime. Thread-local, so
        // a Harness must be constructed and used entirely on one OS thread; re-claiming it is
        // fine for sequentially-used harnesses on the same thread (each `new()` re-installs its
        // own), but concurrent harnesses on one thread are not a supported pattern.
        let async_signal = TestEventSink::new();
        set_event_sink(async_signal.clone());

        Harness {
            env,
            text_context: CosmicTextContext::new(),
            scenes: [scene],
            event_handler: NewEventHandler::new(),
            focus_manager: FocusManager::new(),
            animation_manager: AnimationManager::new(),
            render_instruction_cache: HashMap::new(),
            wgpu_context,
            offscreen,
            dimension,
            scale_factor,
            window_id,
            failures: Vec::new(),
            virtual_now: Instant::now(),
            async_signal,
            test_scope: current_test_scope(),
        }
    }

    /// Runs `f` with `AnimationManager`/`WgpuContext`/`RenderInstructionCache`/`FocusManager`
    /// pushed into the environment for the duration of the call — the same scoping
    /// `RunningApplication` does for the live-window path (`application.rs`), needed because
    /// `NewEventHandler`'s dispatch methods and `OffscreenRenderer` both look these up via
    /// `env.get_mut::<...>()`.
    fn with_env_scope<R>(&mut self, f: impl FnOnce(&mut Environment, &mut [Box<dyn AnyScene>], &mut CosmicTextContext, &mut NewEventHandler, &mut OffscreenRenderer) -> R) -> R {
        let Harness {
            env,
            animation_manager,
            wgpu_context,
            render_instruction_cache,
            focus_manager,
            scenes,
            text_context,
            event_handler,
            offscreen,
            dimension,
            scale_factor,
            ..
        } = self;

        // Mirrors `InitializedWindow::with_env` (the live-window path) -- widgets that read
        // `SceneManager::scale_factor()` at render time (e.g. `Shadow::render`, for its filter's
        // physical-pixel kernel resolution) need to see the *same* scale factor here that this
        // `Harness` was actually constructed with, not silently fall back to 1.0 because no
        // `SceneManager` was ever installed.
        let physical_dimensions = Dimension::new(dimension.width * *scale_factor, dimension.height * *scale_factor);
        let mut scene_manager = SceneManager::new(*scale_factor, physical_dimensions);

        let mut result: Option<R> = None;

        env.with_mut::<AnimationManager>(animation_manager, |env| {
            env.with_mut::<WgpuContext>(wgpu_context, |env| {
                env.with_mut::<RenderInstructionCache>(render_instruction_cache, |env| {
                    env.with_mut::<FocusManager>(focus_manager, |env| {
                        env.with_mut::<SceneManager>(&mut scene_manager, |env| {
                            result = Some(f(env, scenes, text_context, event_handler, offscreen));
                        })
                    })
                })
            })
        });

        result.unwrap()
    }

    /// Keeps whatever `scale_factor`/`msaa` this `Harness` was constructed with (see
    /// `new_with_options`) — only the logical dimension changes.
    pub fn resize(&mut self, dimension: Dimension) {
        self.dimension = dimension;
        let physical = physical_size(dimension, self.scale_factor);
        let scale_factor = self.scale_factor;

        self.with_env_scope(|env, scenes, text_context, _event_handler, offscreen| {
            offscreen.resize(physical, scale_factor, env);

            scenes[0].process_window_event(&WindowEvent::Resize(dimension), &mut WindowEventContext {
                text: text_context,
                image: &mut WGPUImageContext,
                env,
                is_current: &false,
                window_id: &0,
            });
        });
    }

    /// Sets the theme (`Theme::Light`/`Theme::Dark`) the root `ThemeManager` — added to every
    /// `Harness`'s widget tree in `new_with_options`, mirroring `Window::new` — resolves
    /// `EnvironmentColor` values against. Persists across `render()` calls until changed again,
    /// same as `resize`. Uses the same leaked-`Box` pattern as `Application::add_environment_owned`
    /// (`Environment<'static>::insert` requires a `&'static` reference) — harmless for a test
    /// process, since each call leaks one `Theme`-sized allocation.
    pub fn set_theme(&mut self, theme: Theme) {
        self.env.insert::<Theme>(Box::leak(Box::new(theme)));
    }

    /// `position` is in logical units (the same space `find_by_label`'s bounds and `dimension`
    /// use). `NewEventHandler::cursor_moved` takes *physical* pixels and internally converts back
    /// to logical via `carbide_winit::scale_factor(window_id)` — a value looked up from a
    /// process-wide (not per-`Harness`) table keyed by `WindowId`, which every `Harness` shares
    /// since `WindowId::dummy()` always returns the same value. Re-registering this `Harness`'s
    /// own scale factor immediately before the call (rather than only once, back in `new()`)
    /// keeps the write-then-read adjacent on this thread, which is what actually matters: `cargo
    /// test` runs different `#[test]` functions concurrently on separate threads by default, so a
    /// `Harness` with a different scale factor running concurrently on another thread could
    /// otherwise clobber this table between this `Harness`'s construction and its first event.
    pub fn mouse_move(&mut self, position: Position) {
        let window_id = self.window_id;
        let scale_factor = self.scale_factor;
        update_scale_factor(window_id, scale_factor);
        let physical = PhysicalPosition::new(position.x * scale_factor, position.y * scale_factor);

        self.with_env_scope(|env, scenes, text_context, event_handler, _offscreen| {
            event_handler.cursor_moved(physical, window_id, scenes, text_context, &mut WGPUImageContext, env);
        });
        self.refocus();
    }

    pub fn mouse_press(&mut self, button: MouseButton) {
        let window_id = self.window_id;
        self.with_env_scope(|env, scenes, text_context, event_handler, _offscreen| {
            event_handler.mouse_input(button, ElementState::Pressed, window_id, scenes, text_context, &mut WGPUImageContext, env);
        });
        self.refocus();
    }

    pub fn mouse_release(&mut self, button: MouseButton) {
        let window_id = self.window_id;
        self.with_env_scope(|env, scenes, text_context, event_handler, _offscreen| {
            event_handler.mouse_input(button, ElementState::Released, window_id, scenes, text_context, &mut WGPUImageContext, env);
        });
        self.refocus();
    }

    /// Move + press + release at `position` — the recommended pattern for driving the harness,
    /// mirroring real mouse input. For a widget's "held down" state, use `mouse_press`/
    /// `mouse_release` separately instead (this collapses that state).
    pub fn click(&mut self, position: Position) {
        self.mouse_move(position);
        self.mouse_press(MouseButton::Left);
        self.mouse_release(MouseButton::Left);
    }

    /// `delta_x`/`delta_y` are in logical units, for the same reason — and same scale-factor
    /// re-registration — as `mouse_move`.
    pub fn scroll(&mut self, delta_x: f64, delta_y: f64) {
        let window_id = self.window_id;
        let scale_factor = self.scale_factor;
        update_scale_factor(window_id, scale_factor);
        let physical = PhysicalPosition::new(delta_x * scale_factor, delta_y * scale_factor);

        self.with_env_scope(|env, scenes, text_context, event_handler, _offscreen| {
            event_handler.mouse_wheel(MouseScrollDelta::PixelDelta(physical), window_id, scenes, text_context, &mut WGPUImageContext, env);
        });
        self.refocus();
    }

    pub fn key_press(&mut self, key: Key) {
        let window_id = self.window_id;
        self.with_env_scope(|env, scenes, text_context, event_handler, _offscreen| {
            event_handler.keyboard(key.clone(), key.clone(), ElementState::Pressed, window_id, scenes, text_context, &mut WGPUImageContext, env);
        });
        self.refocus();
    }

    pub fn key_release(&mut self, key: Key) {
        let window_id = self.window_id;
        self.with_env_scope(|env, scenes, text_context, event_handler, _offscreen| {
            event_handler.keyboard(key.clone(), key.clone(), ElementState::Released, window_id, scenes, text_context, &mut WGPUImageContext, env);
        });
        self.refocus();
    }

    /// Per-character `key_press`+`key_release`.
    pub fn type_text(&mut self, s: &str) {
        for ch in s.chars() {
            let key = Key::Character(ch.to_string().into());
            self.key_press(key.clone());
            self.key_release(key);
        }
    }

    fn refocus(&mut self) {
        NewEventHandler::handle_refocus(&mut self.scenes, &mut self.focus_manager, &mut self.env);
    }

    /// Drains completions for the `task!`/`SpawnTask`/`Task`-style async API
    /// (`carbide_core::asynchronous::{spawn_task, check_tasks}`) — a plain synchronous function
    /// over thread-local queues, needing no winit/live event loop. Does not see mandelbrot-style
    /// hand-rolled background-thread+channel patterns, since those never enter carbide's async
    /// queues at all; for those, call `render()` after `wait_for_async_event` returns, so the
    /// widget's own polling (typically inside its `Render::render()`) picks up the result.
    pub fn run_async_tasks(&mut self) {
        let mut ctx = AsyncContext {
            text: &mut self.text_context,
            image: &mut WGPUImageContext,
            env: &mut self.env,
        };
        check_tasks(&mut ctx);
    }

    /// Blocks (bounded by `timeout`, no busy-polling) until real background work signals
    /// completion via `carbide_core::asynchronous::get_event_sink().send(CoreEvent::Async)` —
    /// which both the proper async API and hand-rolled patterns like the `mandelbrot` example's
    /// `rayon::spawn` + manual notify already do. Returns whether an event was observed before
    /// the timeout elapsed. This genuinely waits on real wall-clock work finishing on another
    /// thread — not something `advance_time` can fast-forward.
    pub fn wait_for_async_event(&mut self, timeout: Duration) -> bool {
        self.async_signal.wait(timeout)
    }

    /// Builds the full accessibility tree headlessly via `process_accessibility`/
    /// `AccessibilityContext` — plain data, no `accesskit_winit::Adapter`/OS accessibility
    /// bridge needed, the same "reuse an existing plain-data seam" pattern as the event-synthesis
    /// and async-signal seams elsewhere in this harness.
    ///
    /// Runs layout first (`process_update`/`calculate_size`/`set_position`/`position_children`,
    /// mirroring `render()`), rather than relying on the caller having already called `render()`
    /// at least once. Accessibility node bounds are computed from the widget's position/dimension
    /// fields, which default to `(0,0)`/`(0,0)` until a layout pass runs — without this, calling
    /// `find_by_label` before the first `render()` would silently report zero-sized bounds for
    /// every node instead of failing loudly, exactly the kind of footgun this fixture should not
    /// leave for its own callers.
    pub fn accessibility_tree(&mut self) -> Vec<(WidgetId, accesskit::Node)> {
        let dimension = self.dimension;
        let scale_factor = self.scale_factor;

        self.with_env_scope(|env, scenes, text_context, _event_handler, _offscreen| {
            let scene = &mut scenes[0];

            scene.process_update(&mut UpdateContext {
                text: text_context,
                image: &mut WGPUImageContext,
                env,
            });

            scene.calculate_size(dimension, &mut LayoutContext {
                text: text_context,
                image: &mut WGPUImageContext,
                env,
            });

            let child_dimension = scene.dimension();
            scene.set_position(Alignment::Center.position(Position::new(0.0, 0.0), dimension, child_dimension));

            scene.position_children(Rect::new(Position::new(0.0, 0.0), dimension), &mut LayoutContext {
                text: text_context,
                image: &mut WGPUImageContext,
                env,
            });

            let mut tree = accesskit::TreeUpdate {
                nodes: vec![],
                tree: None,
                tree_id: accesskit::TreeId::ROOT,
                focus: accesskit::NodeId(0),
            };
            let mut children = smallvec::SmallVec::<[WidgetId; 8]>::new();

            scene.process_accessibility(&mut AccessibilityContext {
                env,
                nodes: &mut tree,
                parent_id: None,
                children: &mut children,
                hidden: false,
                inherited_label: None,
                inherited_hint: None,
                inherited_value: None,
                inherited_enabled: None,
            });

            tree.nodes.into_iter()
                .map(|(node_id, mut node)| {
                    // Widget-level accessibility code (e.g. `accessibility.rs`) reports bounds in
                    // *physical* pixels, since that's what a real OS accessibility API needs to
                    // position an on-screen focus indicator -- scaling by the now-installed
                    // `SceneManager`'s `scale_factor`. This testing API's own contract (see
                    // `mouse_move`'s doc comment) is that everything -- including bounds returned
                    // here -- stays in the same logical-pixel space as `dimension`, so convert
                    // back down before handing it to a test.
                    if let Some(bounds) = node.bounds() {
                        node.set_bounds(accesskit::Rect {
                            x0: bounds.x0 / scale_factor,
                            y0: bounds.y0 / scale_factor,
                            x1: bounds.x1 / scale_factor,
                            y1: bounds.y1 / scale_factor,
                        });
                    }
                    (WidgetId::from_u32(node_id.0 as u32), node)
                })
                .collect()
        })
    }

    /// Finds a node by its accessible label. The recommended way to derive a click position for
    /// `mouse_move`/`mouse_press`/`mouse_release` (via `node.bounds()`) instead of hardcoding
    /// pixel coordinates that break on unrelated layout changes.
    pub fn find_by_label(&mut self, label: &str) -> Option<(WidgetId, accesskit::Node)> {
        self.accessibility_tree().into_iter().find(|(_, node)| node.label() == Some(label))
    }

    pub fn find_by_role(&mut self, role: accesskit::Role) -> Vec<(WidgetId, accesskit::Node)> {
        self.accessibility_tree().into_iter().filter(|(_, node)| node.role() == role).collect()
    }

    /// Dispatches an accessibility action directly to the node with the given label, via
    /// `process_accessibility_event` — independent of `process_mouse_event`/
    /// `process_keyboard_event`. A separate, narrower tool for verifying that a widget's
    /// accessibility wiring is actually correct (a widget can handle `MouseEvent::Click` without
    /// also handling `Action::Click`, silently breaking screen-reader users) — not a second
    /// general-purpose way to drive tests. The recommended default interaction remains explicit
    /// mouse events at a `find_by_label`-derived position.
    pub fn invoke_accessibility_action(&mut self, label: &str, action: accesskit::Action) {
        let (widget_id, _) = self.find_by_label(label)
            .unwrap_or_else(|| panic!("invoke_accessibility_action: no accessible node with label {label:?}"));

        self.with_env_scope(|env, scenes, _text_context, _event_handler, _offscreen| {
            scenes[0].process_accessibility_event(&AccessibilityEvent {
                action,
                target: widget_id,
                data: &None,
            }, &mut AccessibilityEventContext {
                env,
            });
        });

        // Some accessibility actions (e.g. Focus, or Click on a focusable control) request focus
        // via FocusManager rather than granting it immediately — the same as a mouse/keyboard
        // event handler would, and just like those, the request needs resolving before the
        // caller can observe the final focus state.
        self.refocus();
    }

    /// Advances the fake animation clock by `duration`, with no real wall-clock delay.
    /// `render()` does not implicitly advance time — a test wanting "render, advance 2s, render
    /// again" calls this explicitly between the two `render()` calls. Has no effect on
    /// `TransitionState`-driven widgets, which read `Instant::now()` directly rather than going
    /// through `AnimationManager` — a known, documented limitation of this fixture, not something
    /// this method can work around.
    pub fn advance_time(&mut self, duration: Duration) {
        self.virtual_now += duration;
        self.animation_manager.update_frame_time_at(self.virtual_now);
    }

    pub fn render(&mut self) {
        let dimension = self.dimension;
        let scale_factor = self.scale_factor;

        self.with_env_scope(|env, scenes, text_context, _event_handler, offscreen| {
            let scene = &mut scenes[0];

            scene.process_update(&mut UpdateContext {
                text: text_context,
                image: &mut WGPUImageContext,
                env,
            });

            scene.calculate_size(dimension, &mut LayoutContext {
                text: text_context,
                image: &mut WGPUImageContext,
                env,
            });

            let child_dimension = scene.dimension();
            scene.set_position(Alignment::Center.position(Position::new(0.0, 0.0), dimension, child_dimension));

            scene.position_children(Rect::new(Position::new(0.0, 0.0), dimension), &mut LayoutContext {
                text: text_context,
                image: &mut WGPUImageContext,
                env,
            });

            let mut encoder = {
                let wgpu_context = env.get_mut::<WgpuContext>().unwrap();
                wgpu_context.device.create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("carbide_harness_encoder"),
                })
            };

            offscreen.render_child(scene, dimension, scale_factor, &mut RenderContext {
                render: &mut NoopRenderContext,
                text: text_context,
                image: &mut WGPUImageContext,
                env,
            }, &mut encoder);

            let wgpu_context = env.get_mut::<WgpuContext>().unwrap();
            wgpu_context.queue.submit(Some(encoder.finish()));
        });
    }

    /// Reads the last-rendered frame back from the GPU as an `image::RgbaImage`, via the shared
    /// [`crate::screenshot::capture_texture_to_image`] readback (also used by the live window's
    /// `SceneManager::take_screenshot()` path).
    pub fn pixels(&mut self) -> image::RgbaImage {
        crate::screenshot::capture_texture_to_image(
            &self.wgpu_context.device,
            &self.wgpu_context.queue,
            &self.offscreen.targets[0].texture,
        )
    }

    /// Layout-and-record only, no GPU work at all — the CPU-computed draw-call plan that
    /// `render()` would otherwise hand off for GPU execution. `pub(crate)`: the primary public
    /// entry point is `snapshot_render_instructions`, which formats this and compares it; the raw
    /// `Vec<RenderPass>` isn't exposed directly since `RenderPass` lives in a crate-private
    /// module (nothing outside this crate can currently name it usefully).
    pub(crate) fn render_instructions(&mut self) -> Vec<crate::render_pass_command::RenderPass> {
        let dimension = self.dimension;
        let scale_factor = self.scale_factor;

        self.with_env_scope(|env, scenes, text_context, _event_handler, offscreen| {
            let scene = &mut scenes[0];

            scene.process_update(&mut UpdateContext {
                text: text_context,
                image: &mut WGPUImageContext,
                env,
            });

            scene.calculate_size(dimension, &mut LayoutContext {
                text: text_context,
                image: &mut WGPUImageContext,
                env,
            });

            let child_dimension = scene.dimension();
            scene.set_position(Alignment::Center.position(Position::new(0.0, 0.0), dimension, child_dimension));

            scene.position_children(Rect::new(Position::new(0.0, 0.0), dimension), &mut LayoutContext {
                text: text_context,
                image: &mut WGPUImageContext,
                env,
            });

            offscreen.build_render_passes(scene, dimension, scale_factor, &mut RenderContext {
                render: &mut NoopRenderContext,
                text: text_context,
                image: &mut WGPUImageContext,
                env,
            })
        })
    }

    /// Renders, reads back pixels, and compares against `tests/snapshots/{test}/{name}.png` (see
    /// `current_test_scope`) with the default tolerance. On mismatch, records a failure (does not
    /// panic immediately — see `snapshot_with_options`) so a test with several `snapshot()` calls
    /// runs every one of them.
    pub fn snapshot(&mut self, name: &str) {
        self.snapshot_with_options(name, SnapshotOptions::default());
    }

    pub fn snapshot_with_options(&mut self, name: &str, options: SnapshotOptions) {
        self.render();
        let pixels = self.pixels();
        if let Err(message) = snapshot::compare_and_report(name, &pixels, &options, self.test_scope.as_deref()) {
            self.failures.push(message);
        }
    }

    /// Structural snapshot of the accessibility tree, as JSON (via accesskit's `serde` feature —
    /// no hand-written serialization needed), compared exactly against
    /// `tests/snapshots/{test}/{name}.a11y.json` (see `current_test_scope`). Catches a regression
    /// class pixel snapshots can't: a refactor that leaves pixels unchanged but silently drops a
    /// widget's `role()`/`label()`.
    ///
    /// `WidgetId` is a global, process-wide monotonic counter — its raw value depends on how many
    /// widgets were constructed earlier in the same test binary (other tests, other harnesses),
    /// which `cargo test` does not guarantee is stable across runs. Using it directly as the
    /// snapshot's node identifier would make this flaky for reasons entirely unrelated to the
    /// widget under test. Re-indexed by tree-traversal position instead, which is stable.
    pub fn snapshot_accessibility_tree(&mut self, name: &str) {
        let tree: Vec<(usize, accesskit::Node)> = self.accessibility_tree()
            .into_iter()
            .enumerate()
            .map(|(index, (_, mut node))| {
                node.set_author_id(format!("n{index}"));
                (index, node)
            })
            .collect();

        let json = serde_json::to_string_pretty(&tree).expect("serializing the accessibility tree to JSON should not fail");

        if let Err(message) = snapshot::compare_text_and_report(name, &json, "a11y", self.test_scope.as_deref()) {
            self.failures.push(message);
        }
    }

    /// Structural snapshot of the CPU-computed render-instruction plan (`render_instructions()`),
    /// as JSON, compared exactly against `tests/snapshots/{test}/{name}.render.json` (see
    /// `current_test_scope`). Unlike pixels, this is pure CPU logic with no GPU rasterization
    /// involved, so it's exactly as reproducible on a software rasterizer as on real hardware —
    /// no tolerance-tuning ever needed for this layer.
    ///
    /// `RenderPass`/`RenderPassCommand`/`WGPUBindGroup` don't derive `Serialize` (adding it would
    /// mean adding it across their whole nested type graph — `Rect`, `FilterId`, `ImageId`,
    /// `LayerId`, `WgpuGradient`, `WgpuDashes` — including several `carbide_core` types, for a
    /// benefit specific to this one snapshot), so `render_passes_to_json` below builds the
    /// `serde_json::Value` tree by hand instead, falling back to each inner type's existing
    /// `Debug` output for the handful of fields it doesn't break out explicitly.
    pub fn snapshot_render_instructions(&mut self, name: &str) {
        let instructions = self.render_instructions();
        let json = serde_json::to_string_pretty(&render_passes_to_json(&instructions))
            .expect("serializing the render-instruction plan to JSON should not fail");

        if let Err(message) = snapshot::compare_text_and_report(name, &json, "render", self.test_scope.as_deref()) {
            self.failures.push(message);
        }
    }
}

impl Drop for Harness {
    /// Panics with every accumulated snapshot failure, not just the first — so a test with
    /// several `snapshot()`-family calls reports every mismatch found in one run. Guarded against
    /// panicking during an unrelated unwind (which would abort the process instead of just
    /// failing the test).
    fn drop(&mut self) {
        if !self.failures.is_empty() && !std::thread::panicking() {
            panic!("{}", self.failures.join("\n"));
        }
    }
}
