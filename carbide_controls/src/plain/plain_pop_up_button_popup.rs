use carbide::event::EventId;
use carbide::scene::SceneManager;
use carbide::widget::OverlayManager;
use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::event::{
    KeyboardEvent, KeyboardEventHandler, MouseButton, MouseEvent, MouseEventHandler, KeyboardEventContext, MouseEventContext
};
use carbide_core::layout::{Layout, LayoutContext};
use carbide_core::state::{ReadState, State, StateContract, LocalState};
use carbide_core::widget::{AnyWidget, CommonWidget, Widget, WidgetExt, WidgetId};
use crate::ControlsOverlayKey;
use crate::plain::plain_pop_up_button::PopupButtonKeyCommand;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(MouseEvent, KeyboardEvent, Layout)]
pub struct PlainPopUpButtonPopUp<T, S, M, H, E> where
    T: StateContract + PartialEq,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    H: State<T=Option<usize>>,
    E: ReadState<T=bool>,
{
    id: WidgetId,
    child: Box<dyn AnyWidget>,
    position: Position,
    dimension: Dimension,
    parent_position: LocalState<Position>,
    parent_dimension: LocalState<Dimension>,
    overlay_id: Option<String>,
    #[state] model: M,
    #[state] selected: S,
    #[state] hover_model: H,
    #[state] enabled: E,

    /// The ID of the event causing this to open.
    _event_id: EventId,
}

impl PlainPopUpButtonPopUp<bool, bool, Vec<bool>, Option<usize>, bool> {
    pub fn new<T: StateContract + PartialEq, S: State<T=T>, M: ReadState<T=Vec<T>>, H: State<T=Option<usize>>, E: ReadState<T=bool>>(child: Box<dyn AnyWidget>, hover_model: H, model: M, selected: S, enabled: E, overlay_id: Option<String>, parent_position: LocalState<Position>, parent_dimension: LocalState<Dimension>, event_id: EventId) -> PlainPopUpButtonPopUp<T, S, M, H, E> {
        PlainPopUpButtonPopUp {
            id: WidgetId::new(),
            child,
            position: Default::default(),
            dimension: Default::default(),
            parent_position,
            parent_dimension,
            overlay_id,
            model,
            selected,
            hover_model,
            enabled,
            _event_id: event_id,
        }
    }
}

impl<
    T: StateContract + PartialEq,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    H: State<T=Option<usize>>,
    E: ReadState<T=bool>,
> Layout for PlainPopUpButtonPopUp<T, S, M, H, E> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let new_requested = Dimension::new(self.parent_dimension.value().width, requested_size.height);
        let result = self.child.calculate_size(new_requested, ctx);
        self.set_dimension(result);

        self.dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let alignment = self.alignment();
        let position = *self.parent_position.value();
        let dimension = *self.parent_dimension.value();

        self.set_position(alignment.position(position, dimension, self.dimension));

        let scene_dimensions = ctx.env_stack.get_mut::<SceneManager>()
            .map(|a| a.dimensions())
            .unwrap_or(Dimension::new(600.0, 600.0));

        self.position = self.position
            .min(&Position::new(scene_dimensions.width - self.width(), scene_dimensions.height - self.height()))
            .max(&Position::new(0.0, 0.0));

        let position = self.position();
        let dimension = self.dimension();
        self.child.set_position(alignment.position(position, dimension, self.child.dimension()));
        self.child.position_children(ctx);
    }
}

impl<
    T: StateContract + PartialEq,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    H: State<T=Option<usize>>,
    E: ReadState<T=bool>,
> KeyboardEventHandler for PlainPopUpButtonPopUp<T, S, M, H, E> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        if !*self.enabled.value() {
            OverlayManager::get::<ControlsOverlayKey>(ctx.env_stack, |manager| {
                manager.clear()
            });
            return;
        }

        ctx.prevent_default();

        if event == PopupButtonKeyCommand::Close {
            OverlayManager::get::<ControlsOverlayKey>(ctx.env_stack, |manager| {
                manager.clear()
            })
        } else if event == PopupButtonKeyCommand::Select {
            let value: Option<usize> = self.hover_model.value().clone();
            if let Some(h) = value {
                let value = self.model.value()[h].clone();
                self.selected.set_value(value);
            }

            OverlayManager::get::<ControlsOverlayKey>(ctx.env_stack, |manager| {
                manager.clear()
            })
        } else if event == PopupButtonKeyCommand::Next {
            let value: Option<usize> = self.hover_model.value().clone();
            if let Some(h) = value {
                let new = if h == self.model.value().len() - 1 { 0 } else { h + 1 };
                self.hover_model.set_value(Some(new));
            } else {
                self.hover_model.set_value(Some(0));
            }
        } else if event == PopupButtonKeyCommand::Prev {
            let value: Option<usize> = self.hover_model.value().clone();
            if let Some(h) = value {
                let new = if h == 0 { self.model.value().len() - 1 } else { h - 1 };
                self.hover_model.set_value(Some(new));
            } else {
                self.hover_model.set_value(Some(self.model.value().len() - 1));
            }
        }
    }
}

impl<
    T: StateContract + PartialEq,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    H: State<T=Option<usize>>,
    E: ReadState<T=bool>,
> MouseEventHandler for PlainPopUpButtonPopUp<T, S, M, H, E> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        if !*self.enabled.value() {
            OverlayManager::get::<ControlsOverlayKey>(ctx.env_stack, |manager| {
                manager.clear()
            });
            //self.popup_open.set_value(false);
            return;
        }

        match event {
            MouseEvent::Release { button: MouseButton::Left, position, ..} => {
                if !self.is_inside(*position) {
                    OverlayManager::get::<ControlsOverlayKey>(ctx.env_stack, |manager| {
                        manager.clear()
                    })
                }
            }
            _ => (),
        }
    }
}

impl<
    T: StateContract + PartialEq,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    H: State<T=Option<usize>>,
    E: ReadState<T=bool>,
> CommonWidget for PlainPopUpButtonPopUp<T, S, M, H, E> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}