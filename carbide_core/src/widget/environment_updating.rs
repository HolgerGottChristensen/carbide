use carbide::accessibility::AccessibilityContext;
use carbide::event::{AccessibilityEvent, AccessibilityEventContext};
use carbide::lifecycle::{InitializationContext, Initialize};
use crate::focus::FocusContext;
use carbide_macro::carbide_default_builder2;
use crate::accessibility::Accessibility;
use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::event::{AccessibilityEventHandler, Event, KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler, OtherEventContext, OtherEventHandler, WindowEvent, WindowEventContext, WindowEventHandler};
use crate::focus::Focusable;
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, RenderContext};
use crate::state::{ReadState, StateContract};
use crate::lifecycle::{Update, UpdateContext};
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
        //env.pop();
        todo!()
    }

    fn insert_into_env(&mut self, env: &mut Environment) {
        /*self.value.sync(env);
        env.push(self.key, Box::new(self.value.value().clone()));*/
        todo!()
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

        let alignment = self.alignment();
        let position = self.position();
        let dimension = self.dimension();
        self.child.set_position(alignment.position(position, dimension, self.child.dimension()));
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

impl<C: Widget, T: StateContract, S: ReadState<T=T>> Initialize for EnvUpdating<C, T, S> {
    fn process_initialization(&mut self, ctx: &mut InitializationContext) {
        self.insert_into_env(ctx.env);

        self.child.process_initialization(ctx);

        self.remove_from_env(ctx.env);
    }
}

impl<C: Widget, T: StateContract, S: ReadState<T=T>> Accessibility for EnvUpdating<C, T, S> {
    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        self.insert_into_env(ctx.env);

        self.child.process_accessibility(ctx);

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

impl<C: Widget, T: StateContract, S: ReadState<T=T>> AccessibilityEventHandler for EnvUpdating<C, T, S> {
    fn process_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {
        self.insert_into_env(ctx.env);

        self.child.process_accessibility_event(event, ctx);

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
    fn process_focus_request(&mut self, ctx: &mut FocusContext) {
        self.insert_into_env(ctx.env);

        self.child.process_focus_request(ctx);

        self.remove_from_env(ctx.env);
    }

    fn process_focus_next(&mut self, ctx: &mut FocusContext) {
        self.insert_into_env(ctx.env);

        self.child.process_focus_next(ctx);

        self.remove_from_env(ctx.env);
    }

    fn process_focus_previous(
        &mut self,
        ctx: &mut FocusContext,
    ) {
        self.insert_into_env(ctx.env);

        let focus_child = self.child.process_focus_previous(ctx);

        self.remove_from_env(ctx.env);

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