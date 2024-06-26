use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::event::{Event, KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler, OtherEventContext, OtherEventHandler, WindowEvent, WindowEventContext, WindowEventHandler};
use crate::focus::{Focusable, Refocus};
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, RenderContext};
use crate::state::{ReadState, StateContract};
use crate::update::{Update, UpdateContext};
use crate::widget::{CommonWidget, Widget, WidgetExt, WidgetId};

pub trait EnvKey {
    fn key(&self) -> &'static str;
}

impl EnvKey for &'static str {
    fn key(&self) -> &'static str {
        *self
    }
}

#[derive(Debug, Clone, Widget)]
#[carbide_derive(StateSync)]
pub struct EnvUpdating<C, T, S> where C: Widget, T: StateContract, S: ReadState<T=T> {
    id: WidgetId,
    child: C,
    position: Position,
    dimension: Dimension,
    key: &'static str,
    value: S,
}

impl<C: Widget, T: StateContract, S: ReadState<T=T>> EnvUpdating<C, T, S> {
    #[carbide_default_builder2]
    pub fn new<K: EnvKey>(key: K, value: S, child: C) -> EnvUpdating<C, T, S> {
        EnvUpdating {
            id: WidgetId::new(),
            child,
            position: Position::default(),
            dimension: Dimension::default(),
            key: key.key(),
            value
        }
    }
}

impl<C: Widget, T: StateContract, S: ReadState<T=T>> EnvUpdating<C, T, S> {

    fn remove_from_env(&self, env: &mut Environment) {
        env.pop();
    }

    fn insert_into_env(&mut self, env: &mut Environment) {
        self.value.sync(env);
        env.push(self.key, Box::new(self.value.value().clone()));
    }
}

impl<C: Widget, T: StateContract, S: ReadState<T=T>> Layout for EnvUpdating<C, T, S> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.insert_into_env(ctx.env);

        let chosen = self.child.calculate_size(requested_size, ctx);
        self.set_dimension(chosen);

        self.remove_from_env(ctx.env);
        chosen
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        self.insert_into_env(ctx.env);

        let positioning = self.alignment().positioner();
        let position = self.position();
        let dimension = self.dimension();
        positioning(position, dimension, &mut self.child);
        self.child.position_children(ctx);

        self.remove_from_env(ctx.env);
    }
}

impl<C: Widget, T: StateContract, S: ReadState<T=T>> Update for EnvUpdating<C, T, S> {
    fn process_update(&mut self, ctx: &mut UpdateContext) {
        self.insert_into_env(ctx.env);

        self.child.process_update(ctx);

        self.remove_from_env(ctx.env);
    }
}

impl<C: Widget, T: StateContract, S: ReadState<T=T>> OtherEventHandler for EnvUpdating<C, T, S> {
    fn process_other_event(&mut self, event: &Event, ctx: &mut OtherEventContext) {
        self.insert_into_env(ctx.env);

        self.child.process_other_event(event, ctx);

        self.remove_from_env(ctx.env);
    }
}

impl<C: Widget, T: StateContract, S: ReadState<T=T>> KeyboardEventHandler for EnvUpdating<C, T, S> {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        self.insert_into_env(ctx.env);

        self.child.process_keyboard_event(event, ctx);

        self.remove_from_env(ctx.env);
    }
}

impl<C: Widget, T: StateContract, S: ReadState<T=T>> WindowEventHandler for EnvUpdating<C, T, S> {
    fn process_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        self.insert_into_env(ctx.env);

        self.child.process_window_event(event, ctx);

        self.remove_from_env(ctx.env);
    }
}

impl<C: Widget, T: StateContract, S: ReadState<T=T>> MouseEventHandler for EnvUpdating<C, T, S> {
    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        self.insert_into_env(ctx.env);

        self.child.process_mouse_event(event, ctx);

        self.remove_from_env(ctx.env);
    }
}

impl<C: Widget, T: StateContract, S: ReadState<T=T>> Focusable for EnvUpdating<C, T, S> {
    fn process_focus_request(
        &mut self,
        focus_request: &Refocus,
        env: &mut Environment,
    ) -> bool {
        self.insert_into_env(env);

        let any_focus = self.child.process_focus_request(focus_request, env);

        self.remove_from_env(env);

        any_focus
    }

    fn process_focus_next(
        &mut self,
        focus_request: &Refocus,
        focus_up_for_grab: bool,
        env: &mut Environment,
    ) -> bool {
        self.insert_into_env(env);

        let focus_child = self.child.process_focus_next(focus_request, focus_up_for_grab, env);

        self.remove_from_env(env);

        focus_child
    }

    fn process_focus_previous(
        &mut self,
        focus_request: &Refocus,
        focus_up_for_grab: bool,
        env: &mut Environment,
    ) -> bool {
        self.insert_into_env(env);

        let focus_child = self.child.process_focus_previous(focus_request, focus_up_for_grab, env);

        self.remove_from_env(env);

        focus_child
    }
}

impl<C: Widget, T: StateContract, S: ReadState<T=T>> Render for EnvUpdating<C, T, S> {
    fn render(&mut self, context: &mut RenderContext) {
        self.insert_into_env(context.env);

        self.foreach_child_mut(&mut |child| {
            child.render(context);
        });

        self.remove_from_env(context.env);
    }
}

impl<C: Widget, T: StateContract, S: ReadState<T=T>> CommonWidget for EnvUpdating<C, T, S> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}

impl<C: Widget, T: StateContract, S: ReadState<T=T>> WidgetExt for EnvUpdating<C, T, S> {}
