use core::fmt;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use dyn_clone::DynClone;

use crate::event::event::Event;
use crate::event_handler::{KeyboardEvent, MouseEvent, WidgetEvent};
use crate::focus::{Focus, Focusable, Refocus};
use crate::prelude::*;
use crate::widget::{EnvUpdating, Frame, Offset};
use crate::widget::primitive::border::Border;
use crate::widget::primitive::clip::Clip;
use crate::widget::primitive::environment_updating::EnvironmentStateContainer;
use crate::widget::primitive::hidden::Hidden;
use crate::widget::primitive::padding::Padding;
use crate::widget::types::edge_insets::EdgeInsets;

pub trait Widget<S>: Event<S> + Layout<S> + Render<S> + Focusable<S> + DynClone where S: GlobalState {}

//impl<S, T> Widget<S> for T where T: Event<S> + Layout<S> + Render<S> + DynClone {}

impl<S: GlobalState> Widget<S> for Box<dyn Widget<S>> {}

dyn_clone::clone_trait_object!(<S> Widget<S>);

pub trait WidgetExt<GS: GlobalState>: Widget<GS> + Sized + 'static {
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

impl<S: GlobalState> CommonWidget<S> for Box<dyn Widget<S>> {
    fn get_id(&self) -> Uuid {
        self.deref().get_id()
    }

    fn set_id(&mut self, id: Uuid) {
        self.deref_mut().set_id(id);
    }

    fn get_flag(&self) -> Flags {
        self.deref().get_flag()
    }

    fn get_children(&self) -> WidgetIter<S> {
        self.deref().get_children()
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        self.deref_mut().get_children_mut()
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
        self.deref_mut().get_proxied_children()
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<S> {
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


impl<S: GlobalState> Event<S> for Box<dyn Widget<S>> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment<S>, global_state: &mut S) {
        self.deref_mut().handle_mouse_event(event, consumed, env, global_state)
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<S>, global_state: &mut S) {
        self.deref_mut().handle_keyboard_event(event, env, global_state)
    }

    fn handle_other_event(&mut self, event: &WidgetEvent, env: &mut Environment<S>, global_state: &mut S) {
        self.deref_mut().handle_other_event(event, env, global_state)
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment<S>, global_state: &mut S) {
        self.deref_mut().process_mouse_event(event, consumed, env, global_state)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<S>, global_state: &mut S) {
        self.deref_mut().process_keyboard_event(event, env, global_state)
    }

    fn process_other_event(&mut self, event: &WidgetEvent, env: &mut Environment<S>, global_state: &mut S) {
        self.deref_mut().process_other_event(event, env, global_state)
    }
}

impl<S: GlobalState> StateSync<S> for Box<dyn Widget<S>> {
    fn insert_local_state(&self, env: &mut Environment<S>) {
        self.deref().insert_local_state(env)
    }

    fn update_all_widget_state(&mut self, env: &mut Environment<S>, global_state: &S) {
        self.deref_mut().update_all_widget_state(env, global_state)
    }

    fn update_local_widget_state(&mut self, env: &Environment<S>) {
        self.deref_mut().update_local_widget_state(env)
    }

    fn sync_state(&mut self, env: &mut Environment<S>, global_state: &S) {
        self.deref_mut().sync_state(env, global_state)
    }
}

impl<GS: GlobalState> Layout<GS> for Box<dyn Widget<GS>> {
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

impl<GS: GlobalState> Render<GS> for Box<dyn Widget<GS>> {
    fn get_primitives(&mut self, env: &Environment<GS>, global_state: &GS) -> Vec<Primitive> {
        self.deref_mut().get_primitives(env, global_state)
    }
}

impl<GS: GlobalState> Focusable<GS> for Box<dyn Widget<GS>> {
    fn focus_retrieved(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment<GS>, global_state: &mut GS) {
        self.deref_mut().focus_retrieved(event, focus_request, env, global_state)
    }

    fn focus_dismissed(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment<GS>, global_state: &mut GS) {
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

    fn process_focus_request(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment<GS>, global_state: &mut GS) -> bool {
        self.deref_mut().process_focus_request(event, focus_request, env, global_state)
    }

    fn process_focus_next(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment<GS>, global_state: &mut GS) -> bool {
        self.deref_mut().process_focus_next(event, focus_request, focus_up_for_grab, env, global_state)
    }

    fn process_focus_previous(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment<GS>, global_state: &mut GS) -> bool {
        self.deref_mut().process_focus_previous(event, focus_request, focus_up_for_grab, env, global_state)
    }
}


impl<S: GlobalState> Debug for dyn Widget<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Widget: {}", self.get_id())
    }
}