use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use accesskit::{Node, Role};
use dyn_clone::{clone_box, DynClone};
use carbide::accessibility::AccessibilityContext;
use carbide::environment::EnvironmentStack;
use carbide::event::{AccessibilityEvent, AccessibilityEventContext};
use carbide::lifecycle::InitializationContext;
use carbide::widget::Identifiable;
use crate::accessibility::Accessibility;
use crate::draw::{Alignment, Dimension, Position};
use crate::environment::Environment;
use crate::event::{AccessibilityEventHandler, Event, EventHandler, KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler, OtherEventContext, OtherEventHandler, WindowEvent, WindowEventContext, WindowEventHandler};
use crate::flags::WidgetFlag;
use crate::focus::{Focus, Focusable, FocusContext};
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, RenderContext};
use crate::state::{StateContract, StateSync};
use crate::lifecycle::{Initialize, Update, UpdateContext};
use crate::widget::{CommonWidget, IntoWidget, WidgetExt, WidgetId, WidgetSync};

pub trait AnyWidget: EventHandler + Initialize + Update + Accessibility + Layout + Render + Focusable + DynClone + Debug + 'static {
    fn as_widget(&self) -> &dyn AnyWidget;
    fn as_widget_mut(&mut self) -> &mut dyn AnyWidget;
}

impl dyn AnyWidget {
    pub fn boxed(&self) -> Box<dyn AnyWidget> {
        clone_box(self)
    }
}

dyn_clone::clone_trait_object!(AnyWidget);

pub trait Widget: AnyWidget + WidgetExt + Clone + private::Sealed {}

impl<T> Widget for T where T: AnyWidget + WidgetExt + Clone {}

mod private {
    use crate::widget::AnyWidget;

    // This disallows implementing Widget manually, and requires something to implement
    // AnyWidget to implement Widget.
    pub trait Sealed {}

    impl<T> Sealed for T where T: AnyWidget {}
}

// ---------------------------------------------------
//  Implement Widget for Box dyn Widget
// ---------------------------------------------------

impl AnyWidget for Box<dyn AnyWidget> {
    fn as_widget(&self) -> &dyn AnyWidget {
        self
    }

    fn as_widget_mut(&mut self) -> &mut dyn AnyWidget {
        self
    }
}

impl WidgetExt for Box<dyn AnyWidget> {}

impl<T: AnyWidget + ?Sized + Identifiable> Identifiable for Box<T> {
    fn id(&self) -> WidgetId {
        self.deref().id()
    }
}

impl<T: AnyWidget + ?Sized> CommonWidget for Box<T> {
    fn flag(&self) -> WidgetFlag {
        self.deref().flag()
    }

    fn alignment(&self) -> Alignment {
        self.deref().alignment()
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        self.deref().foreach_child(f)
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.deref_mut().foreach_child_mut(f)
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.deref_mut().foreach_child_rev(f)
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.deref_mut().foreach_child_direct(f)
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.deref_mut().foreach_child_direct_rev(f)
    }

    fn position(&self) -> Position {
        self.deref().position()
    }

    fn set_position(&mut self, position: Position) {
        self.deref_mut().set_position(position)
    }

    fn get_focus(&self) -> Focus {
        self.deref().get_focus()
    }

    fn set_focus(&mut self, focus: Focus) {
        self.deref_mut().set_focus(focus)
    }

    fn flexibility(&self) -> u32 {
        self.deref().flexibility()
    }

    fn dimension(&self) -> Dimension {
        self.deref().dimension()
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.deref_mut().set_dimension(dimension)
    }
}

impl<T: AnyWidget + ?Sized> MouseEventHandler for Box<T> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        self.deref_mut().handle_mouse_event(event, ctx)
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        self.deref_mut().process_mouse_event(event, ctx)
    }
}

impl<T: AnyWidget + ?Sized> KeyboardEventHandler for Box<T> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        self.deref_mut().handle_keyboard_event(event, ctx)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        self.deref_mut().process_keyboard_event(event, ctx)
    }
}

impl<T: AnyWidget + ?Sized> WindowEventHandler for Box<T> {
    fn handle_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        self.deref_mut().handle_window_event(event, ctx)
    }

    fn process_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        self.deref_mut().process_window_event(event, ctx)
    }
}

impl<T: AnyWidget + ?Sized> OtherEventHandler for Box<T> {
    fn handle_other_event(&mut self, _event: &Event, ctx: &mut OtherEventContext) {
        self.deref_mut().handle_other_event(_event, ctx)
    }

    fn process_other_event(&mut self, event: &Event, ctx: &mut OtherEventContext) {
        self.deref_mut().process_other_event(event, ctx)
    }
}

impl<T: AnyWidget + ?Sized> AccessibilityEventHandler for Box<T> {
    fn handle_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {
        self.deref_mut().handle_accessibility_event(event, ctx)
    }

    fn process_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {
        self.deref_mut().process_accessibility_event(event, ctx)
    }
}

impl<T: WidgetSync + ?Sized> WidgetSync for Box<T> {
    fn sync(&mut self, env: &mut EnvironmentStack) {
        self.deref_mut().sync(env);
    }
}

impl<T: AnyWidget + ?Sized> Update for Box<T> {
    fn update(&mut self, ctx: &mut UpdateContext) {
        self.deref_mut().update(ctx);
    }

    fn process_update(&mut self, ctx: &mut UpdateContext) {
        self.deref_mut().process_update(ctx);
    }
}

impl<T: AnyWidget + ?Sized> Initialize for Box<T> {
    fn initialize(&mut self, ctx: &mut InitializationContext) {
        self.deref_mut().initialize(ctx)
    }

    fn process_initialization(&mut self, ctx: &mut InitializationContext) {
        self.deref_mut().process_initialization(ctx)
    }
}

impl<T: AnyWidget + ?Sized> Accessibility for Box<T> {
    fn accessibility(&mut self, builder: &mut Node, env: &mut Environment) {
        self.deref_mut().accessibility(builder, env);
    }

    fn role(&self) -> Option<Role> {
        self.deref().role()
    }

    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        self.deref_mut().process_accessibility(ctx)
    }
}

impl<T: AnyWidget + ?Sized> Layout for Box<T> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.deref_mut().calculate_size(requested_size, ctx)
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        self.deref_mut().position_children(ctx)
    }
}

impl<T: AnyWidget + ?Sized> Render for Box<T> {
    fn render(&mut self, context: &mut RenderContext) {
        self.deref_mut().render(context)
    }
}

impl<T: AnyWidget + ?Sized> Focusable for Box<T> {
    fn process_focus_request(&mut self, ctx: &mut FocusContext) {
        self.deref_mut().process_focus_request(ctx)
    }

    fn process_focus_next(&mut self, ctx: &mut FocusContext) {
        self.deref_mut().process_focus_next(ctx)
    }

    fn process_focus_previous(&mut self, ctx: &mut FocusContext) {
        self.deref_mut().process_focus_previous(ctx)
    }
}

