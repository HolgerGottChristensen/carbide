use core::fmt;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use dyn_clone::DynClone;

use crate::event::event::Event;
use crate::event_handler::{KeyboardEvent, MouseEvent, WidgetEvent};
use crate::focus::{Focus, Focusable, Refocus};
use crate::prelude::*;
use crate::prelude::global_state::GlobalStateContainer;
use crate::widget::{EnvUpdating, Frame, Offset};
use crate::widget::primitive::border::Border;
use crate::widget::primitive::clip::Clip;
use crate::widget::primitive::environment_updating::EnvironmentStateContainer;
use crate::widget::primitive::hidden::Hidden;
use crate::widget::primitive::padding::Padding;
use crate::widget::render::RenderProcessor;
use crate::widget::types::edge_insets::EdgeInsets;

pub trait Widget<GS>: Event<GS> + Layout<GS> + Render<GS> + RenderProcessor<GS> + Focusable<GS> + DynClone where GS: GlobalStateContract {}

//impl<S, T> Widget<S> for T where T: Event<S> + Layout<S> + Render<S> + DynClone {}

impl<GS: GlobalStateContract> Widget<GS> for Box<dyn Widget<GS>> {}

dyn_clone::clone_trait_object!(<S> Widget<S>);

pub trait WidgetExt<GS: GlobalStateContract>: Widget<GS> + Sized + 'static {
    fn frame<K1: Into<F64State<GS>>, K2: Into<F64State<GS>>>(self, width: K1, height: K2) -> Box<Frame<GS>> {
        Frame::init(width.into(), height.into(), Box::new(self))
    }

    fn frame_width(self, width: F64State<GS>) -> Box<Frame<GS>> {
        Frame::init_width(width, Box::new(self))
    }

    fn padding<E: Into<EdgeInsets>>(self, edge_insets: E) -> Box<Padding<GS>> {
        Padding::init(edge_insets.into(), Box::new(self))
    }
    fn clip(self) -> Box<Clip<GS>> {
        Clip::new(Box::new(self))
    }

    fn hidden(self) -> Box<Hidden<GS>> {
        Hidden::new(Box::new(self))
    }

    fn offset<K1: Into<F64State<GS>>, K2: Into<F64State<GS>>>(self, offset_x: K1, offset_y: K2) -> Box<Offset<GS>> {
        Offset::new(offset_x.into(), offset_y.into(), Box::new(self))
    }

    fn border(self) -> Box<Border<GS>> {
        Border::initialize(Box::new(self))
    }

    fn foreground_color<C: Into<ColorState<GS>>>(self, color: C) -> Box<EnvUpdating<GS>> {
        let mut e = EnvUpdating::new(Box::new(self));
        e.add(EnvironmentStateContainer::Color { key: EnvironmentColor::Label, value: color.into() });

        e
    }

    fn accent_color<C: Into<ColorState<GS>>>(self, color: C) -> Box<EnvUpdating<GS>> {
        let mut e = EnvUpdating::new(Box::new(self));
        e.add(EnvironmentStateContainer::Color { key: EnvironmentColor::Accent, value: color.into() });

        e
    }
}

//This does not currently work with intellisense
//impl<T> WidgetExt for T where T: Widget + 'static {}

impl<GS: GlobalStateContract> CommonWidget<GS> for Box<dyn Widget<GS>> {
    fn get_id(&self) -> Uuid {
        self.deref().get_id()
    }

    fn set_id(&mut self, id: Uuid) {
        self.deref_mut().set_id(id);
    }

    fn get_flag(&self) -> Flags {
        self.deref().get_flag()
    }

    fn get_children(&self) -> WidgetIter<GS> {
        self.deref().get_children()
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<GS> {
        self.deref_mut().get_children_mut()
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<GS> {
        self.deref_mut().get_proxied_children()
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<GS> {
        self.deref_mut().get_proxied_children_rev()
    }

    fn get_position(&self) -> Dimensions {
        self.deref().get_position()
    }

    fn set_position(&mut self, position: Dimensions) {
        self.deref_mut().set_position(position)
    }

    fn get_dimension(&self) -> Dimensions {
        self.deref().get_dimension()
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.deref_mut().set_dimension(dimensions)
    }
}


impl<GS: GlobalStateContract> Event<GS> for Box<dyn Widget<GS>> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment<GS>) {
        self.deref_mut().handle_mouse_event(event, consumed, env)
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<GS>) {
        self.deref_mut().handle_keyboard_event(event, env)
    }

    fn handle_other_event(&mut self, event: &WidgetEvent, env: &mut Environment<GS>) {
        self.deref_mut().handle_other_event(event, env)
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) {
        self.deref_mut().process_mouse_event(event, consumed, env, global_state)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) {
        self.deref_mut().process_keyboard_event(event, env, global_state)
    }

    fn process_other_event(&mut self, event: &WidgetEvent, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) {
        self.deref_mut().process_other_event(event, env, global_state)
    }
}

impl<GS: GlobalStateContract> StateSync<GS> for Box<dyn Widget<GS>> {
    fn capture_state(&mut self, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) {
        self.deref_mut().capture_state(env, global_state);
    }

    fn release_state(&mut self, env: &mut Environment<GS>) {
        self.deref_mut().release_state(env)
    }
}

impl<GS: GlobalStateContract> Layout<GS> for Box<dyn Widget<GS>> {
    fn flexibility(&self) -> u32 {
        self.deref().flexibility()
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment<GS>) -> Dimensions {
        self.deref_mut().calculate_size(requested_size, env)
    }

    fn position_children(&mut self) {
        self.deref_mut().position_children()
    }
}

impl<GS: GlobalStateContract> RenderProcessor<GS> for Box<dyn Widget<GS>> {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) {
        self.deref_mut().process_get_primitives(primitives, env, global_state);
    }
}

impl<GS: GlobalStateContract> Render<GS> for Box<dyn Widget<GS>> {
    fn get_primitives(&mut self, env: &mut Environment<GS>) -> Vec<Primitive> {
        self.deref_mut().get_primitives(env)
    }
}

impl<GS: GlobalStateContract> Focusable<GS> for Box<dyn Widget<GS>> {
    fn focus_retrieved(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) {
        self.deref_mut().focus_retrieved(event, focus_request, env, global_state)
    }

    fn focus_dismissed(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) {
        self.deref_mut().focus_dismissed(event, focus_request, env, global_state)
    }

    fn get_focus(&self) -> Focus {
        self.deref().get_focus()
    }

    fn set_focus(&mut self, focus: Focus) {
        self.deref_mut().set_focus(focus)
    }

    fn set_focus_and_request(&mut self, focus: Focus, env: &mut Environment<GS>) {
        self.deref_mut().set_focus_and_request(focus, env)
    }

    fn process_focus_request(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) -> bool {
        self.deref_mut().process_focus_request(event, focus_request, env, global_state)
    }

    fn process_focus_next(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) -> bool {
        self.deref_mut().process_focus_next(event, focus_request, focus_up_for_grab, env, global_state)
    }

    fn process_focus_previous(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) -> bool {
        self.deref_mut().process_focus_previous(event, focus_request, focus_up_for_grab, env, global_state)
    }
}


impl<S: GlobalStateContract> Debug for dyn Widget<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Widget: {}", self.get_id())
    }
}