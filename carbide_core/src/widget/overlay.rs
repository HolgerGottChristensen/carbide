use std::cell::RefCell;
use std::rc::Rc;

use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::event::{KeyboardEvent, MouseEvent, WidgetEvent};
use crate::event::{KeyboardEventHandler, MouseEventHandler, OtherEventHandler};
use crate::focus::Focusable;
use crate::focus::Refocus;
use crate::layout::Layout;
use crate::render::{Primitive, RenderContext};
use crate::render::Render;
use crate::state::{AnyState, LocalState, ReadState, State, StateSync, TState};
use crate::state::NewStateSync;
use crate::widget::{CommonWidget, Widget, WidgetExt, WidgetId};

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone)]
pub struct Overlay {
    id: WidgetId,
    child: Rc<RefCell<Box<dyn Widget>>>,
    showing: TState<bool>,
    position: TState<Position>,
    dimension: TState<Dimension>,
}

impl Overlay {
    // We do not need to return this in a box, because the overlay widgets should only
    pub fn new(child: Box<dyn Widget>) -> Self {
        Overlay {
            id: WidgetId::new(),
            child: Rc::new(RefCell::new(child)),
            showing: LocalState::new(false),
            position: LocalState::new(Position::new(0.0, 0.0)),
            dimension: LocalState::new(Dimension::new(100.0, 100.0)),
        }
    }

    pub fn showing(mut self, showing: impl Into<TState<bool>>) -> Self {
        self.showing = showing.into();
        self
    }

    pub fn is_showing(&self) -> bool {
        *self.showing.value()
    }

    pub fn set_showing(&mut self, val: bool) {
        self.showing.set_value_dyn(val);
    }

}

impl CommonWidget for Overlay {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn foreach_child<'a>(&'a self, _f: &mut dyn FnMut(&'a dyn Widget)) {
        panic!("Trying to foreach_child of an overlay");
        /*if self.child.borrow().is_ignore() {
            return;
        }

        if self.child.borrow().is_proxy() {
            self.child.borrow().foreach_child(f);
            return;
        }

        f(self.child.borrow().deref());*/
    }

    fn foreach_child_mut<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn Widget)) {
        panic!("Trying to foreach_child_mut of an overlay");
        /*if self.child.borrow().is_ignore() {
            return;
        }

        if self.child.borrow().is_proxy() {
            self.child.get_mut().foreach_child_mut(f);
            return;
        }

        f(self.child.borrow_mut().deref_mut());*/
    }

    fn foreach_child_rev<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn Widget)) {
        panic!("Trying to foreach_child_rev of an overlay");
        /*if self.child.borrow().is_ignore() {
            return;
        }

        if self.child.borrow().is_proxy() {
            self.child.borrow_mut().foreach_child_rev(f);
            return;
        }

        f(self.child.borrow_mut().deref_mut());*/
    }

    fn foreach_child_direct<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn Widget)) {
        panic!("Trying to foreach_child_direct of an overlay");
        //f(self.child.borrow_mut().deref_mut());
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn Widget)) {
        panic!("Trying to foreach_child_direct_rev of an overlay");
        //f(self.child.borrow_mut().deref_mut());
    }

    fn position(&self) -> Position {
        *self.position.value()
    }

    fn set_position(&mut self, position: Position) {
        *self.position.value_mut() = position;
    }

    fn dimension(&self) -> Dimension {
        *self.dimension.value()
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        *self.dimension.value_mut() = dimension;
    }
}

impl Render for Overlay {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        self.child.borrow_mut().render(context, env);
    }

    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        self.child.borrow_mut().process_get_primitives(primitives, env)
    }
}

impl MouseEventHandler for Overlay {
    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment) {
        self.capture_state(env);

        self.child.borrow_mut().process_mouse_event(event, consumed, env)
    }
}

impl OtherEventHandler for Overlay {
    fn process_other_event(&mut self, event: &WidgetEvent, env: &mut Environment) {
        self.capture_state(env);

        self.child.borrow_mut().process_other_event(event, env)
    }
}
impl KeyboardEventHandler for Overlay {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        self.capture_state(env);

        self.child.borrow_mut().process_keyboard_event(event, env)
    }
}

impl Focusable for Overlay {
    fn process_focus_next(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment) -> bool {
        self.release_state(env);

        self.child.borrow_mut().process_focus_next(event, focus_request, focus_up_for_grab, env)
    }

    fn process_focus_previous(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment) -> bool {
        self.release_state(env);

        self.child.borrow_mut().process_focus_previous(event, focus_request, focus_up_for_grab, env)
    }

    fn process_focus_request(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment) -> bool {
        self.release_state(env);

        self.child.borrow_mut().process_focus_request(event, focus_request, env)
    }
}

impl Layout for Overlay {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {

        let chosen = self.child.borrow_mut().calculate_size(requested_size, env);

        self.set_dimension(chosen);
        chosen
    }

    /// This method positions the children of the widget. When positioning, we use the alignment of
    /// the widget to position. The default alignment is Center.
    /// The default behavior is to position the first child using the alignment of the widget. If
    /// no child are present the default is a no-op.
    fn position_children(&mut self, env: &mut Environment) {
        let positioning = self.alignment().positioner();
        let position = self.position();
        let dimension = self.dimension();

        positioning(position, dimension, &mut **self.child.borrow_mut());
        self.child.borrow_mut().position_children(env);
    }
}

impl StateSync for Overlay {
    fn capture_state(&mut self, env: &mut Environment) {
        self.showing.sync(env);
        self.position.sync(env);
        self.dimension.sync(env);
    }

    fn release_state(&mut self, env: &mut Environment) {
        self.showing.sync(env);
        self.position.sync(env);
        self.dimension.sync(env);
    }
}

impl Widget for Overlay {}

impl WidgetExt for Overlay {}
