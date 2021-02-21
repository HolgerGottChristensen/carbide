use crate::prelude::*;
use crate::event::event::Event;
use dyn_clone::DynClone;
use crate::widget::{Frame, Offset};
use crate::widget::primitive::padding::Padding;
use crate::widget::primitive::hidden::Hidden;
use crate::widget::primitive::clip::Clip;
use crate::widget::primitive::edge_insets::EdgeInsets;
use std::ops::{Deref, DerefMut};
use crate::event_handler::{MouseEvent, KeyboardEvent, WidgetEvent};
use core::fmt;
use std::fmt::Debug;
use crate::widget::primitive::border::Border;
use crate::focus::{Focusable, Focus, Refocus};

pub trait Widget<S>: Event<S> + Layout<S> + Render<S> + Focusable<S> + DynClone where S: GlobalState {}

//impl<S, T> Widget<S> for T where T: Event<S> + Layout<S> + Render<S> + DynClone {}

impl<S: GlobalState> Widget<S> for Box<dyn Widget<S>> {}

dyn_clone::clone_trait_object!(<S> Widget<S>);

pub trait WidgetExt<GS: GlobalState>: Widget<GS> + Sized + 'static {
    fn frame(self, width: Box<dyn State<f64, GS>>, height: Box<dyn State<f64, GS>>) -> Box<Frame<GS>> {
        Frame::init(width, height, Box::new(self))
    }

    fn frame_width(self, width: Box<dyn State<f64, GS>>) -> Box<Frame<GS>> {
        Frame::init_width(width, Box::new(self))
    }

    fn padding(self, edge_insets: EdgeInsets) -> Box<Padding<GS>> {
        Padding::init(edge_insets, Box::new(self))
    }
    fn clip(self) -> Box<Clip<GS>> {
        Clip::new(Box::new(self))
    }

    fn hidden(self) -> Box<Hidden<GS>> {
        Hidden::new(Box::new(self))
    }

    fn offset(self, offset_x: CommonState<f64,GS>, offset_y: CommonState<f64,GS>) -> Box<Offset<GS>> {
        Offset::new(offset_x, offset_y, Box::new(self))
    }

    fn border(self) -> Box<Border<GS>> {
        Border::initialize(Box::new(self))
    }
}

//This does not currently work with intellisense
//impl<T> WidgetExt for T where T: Widget + 'static {}

impl<S: GlobalState> CommonWidget<S> for Box<dyn Widget<S>> {
    fn get_id(&self) -> Uuid {
        self.deref().get_id()
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

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        self.deref_mut().handle_other_event(event)
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

    fn update_all_widget_state(&mut self, env: &Environment<S>, global_state: &S) {
        self.deref_mut().update_all_widget_state(env, global_state)
    }

    fn update_local_widget_state(&mut self, env: &Environment<S>) {
        self.deref_mut().update_local_widget_state(env)
    }

    fn sync_state(&mut self, env: &mut Environment<S>, global_state: &S) {
        self.deref_mut().sync_state(env, global_state)
    }
}

impl<S: GlobalState> Layout<S> for Box<dyn Widget<S>> {
    fn flexibility(&self) -> u32 {
        self.deref().flexibility()
    }

    fn calculate_size(&mut self, requested_size: [f64; 2], env: &Environment<S>) -> [f64; 2] {
        self.deref_mut().calculate_size(requested_size, env)
    }

    fn position_children(&mut self) {
        self.deref_mut().position_children()
    }
}

impl<S: GlobalState> Render<S> for Box<dyn Widget<S>> {
    fn get_primitives(&mut self, fonts: &text::font::Map) -> Vec<Primitive> {
        self.deref_mut().get_primitives(fonts)
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
}


impl<S: GlobalState> Debug for dyn Widget<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Widget: {}", self.get_id())
    }
}