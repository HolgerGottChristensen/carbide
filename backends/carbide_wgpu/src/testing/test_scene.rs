use carbide_core::draw::{Dimension, Position};
use carbide_core::event::{KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler, WindowEvent, WindowEventContext, WindowEventHandler};
use carbide_core::scene::AnyScene;
use carbide_core::widget::{CommonWidget, Widget, WidgetId};
use carbide_core::CommonWidgetImpl;

/// Adapts any widget into something `carbide_winit::NewEventHandler`'s event-dispatch methods
/// (which take `&mut [Box<dyn AnyScene>]`) can drive, without pulling in the winit/`Window`
/// scaffolding a real application uses. Mirrors `carbide_core::widget::{Empty, Hidden}`'s
/// derive-macro pattern: the derive's default `Update`/`Layout`/`Render`/`EventHandler` impls
/// already sync-then-recurse into `foreach_child`, so `child` participates for free.
///
/// `MouseEvent`/`KeyboardEvent`/`WindowEvent` are excluded from that default pass-through and
/// hand-written below: their trait-default `process_*_event` bodies gate `handle_*_event` on
/// `*ctx.is_current`, and every real dispatch path (`carbide_winit::NewEventHandler`) always
/// passes `is_current: &false` — it's the *scene* (normally `Window::process_mouse_event` et al.)
/// that corrects this to `true` for whichever window is actually current. Without an equivalent
/// correction here, a `Harness`'s widgets would silently never receive a single mouse or keyboard
/// event: `handle_mouse_event`/`handle_keyboard_event` would just never run, while layout/render/
/// accessibility dispatch (which don't gate on `is_current`) would look completely normal. Since
/// a `Harness` only ever has one scene, it's always current — no window-id comparison needed.
#[derive(Clone, Debug, Widget)]
#[carbide_exclude(MouseEvent, KeyboardEvent, WindowEvent)]
pub(crate) struct TestScene<C> where C: Widget {
    #[id] id: WidgetId,
    child: C,
    position: Position,
    dimension: Dimension,
}

impl<C: Widget> MouseEventHandler for TestScene<C> {
    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        self.child.process_mouse_event(event, &mut MouseEventContext {
            text: ctx.text,
            image: ctx.image,
            is_current: &true,
            window_id: ctx.window_id,
            consumed: ctx.consumed,
            env: ctx.env,
        });
    }
}

impl<C: Widget> KeyboardEventHandler for TestScene<C> {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        self.child.process_keyboard_event(event, &mut KeyboardEventContext {
            text: ctx.text,
            image: ctx.image,
            env: ctx.env,
            is_current: &true,
            window_id: ctx.window_id,
            prevent_default: ctx.prevent_default,
        });
    }
}

impl<C: Widget> WindowEventHandler for TestScene<C> {
    fn process_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        self.child.process_window_event(event, &mut WindowEventContext {
            text: ctx.text,
            image: ctx.image,
            env: ctx.env,
            is_current: &true,
            window_id: ctx.window_id,
        });
    }
}

impl<C: Widget> TestScene<C> {
    pub(crate) fn new(child: C, dimension: Dimension) -> Self {
        TestScene {
            id: WidgetId::new(),
            child,
            position: Position::origin(),
            dimension,
        }
    }
}

impl<C: Widget> CommonWidget for TestScene<C> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension);
}

impl<C: Widget> AnyScene for TestScene<C> {
    fn request_redraw(&self) -> bool {
        false
    }

    fn has_application_focus(&self) -> bool {
        // So Tab-focus-cycling/IME behave like a real focused window during tests.
        true
    }

    fn is_daemon(&self) -> bool {
        false
    }
}
