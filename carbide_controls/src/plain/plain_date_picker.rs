use std::fmt::{Debug, Formatter};

use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor, WidgetTransferAction};
use carbide_core::event::{
    KeyboardEvent, KeyboardEventHandler, MouseEvent, MouseEventHandler, KeyboardEventContext, MouseEventContext
};
use carbide_core::flags::WidgetFlag;
use carbide_core::focus::{Focus, Refocus};
use carbide_core::state::{AnyReadState, AnyState, LocalState, Map1, ReadState, ReadStateExtNew, State, StateExtNew};
use carbide_core::update::{Update, UpdateContext};
use carbide_core::widget::*;

use crate::plain_calendar::DateSelection;
use crate::PlainCalendar;

#[derive(Clone, Widget)]
#[carbide_exclude(MouseEvent, KeyboardEvent, Update)]
pub struct PlainDatePicker<F, E>
where
    F: State<T=Focus>,
    E: ReadState<T=bool>,
{
    // Default fields
    id: WidgetId,
    position: LocalState<Position>,
    dimension: LocalState<Dimension>,
    #[state] focus: F,
    #[state] enabled: E,

    // Delegates
    //text_delegate: TextDelegateGenerator,
    delegate: DelegateGenerator, // Used to generate the control
    popup: PopupDelegateGenerator,

    child: Box<dyn AnyWidget>,

    selected: DateSelection,
}

impl PlainDatePicker<Focus, bool> {
    pub fn new(selection: impl Into<DateSelection>) -> PlainDatePicker<LocalState<Focus>, bool> {
        let focus = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            selection.into(),
            focus,
            true,
            //|t| Map1::read_map(t, |s| format!("{:?}", s)).as_dyn_read(),
            default_delegate,
            default_popup_delegate
        )
    }
}


impl<F: State<T=Focus>, E: ReadState<T=bool>> PlainDatePicker<F, E> {
    fn open_popup(&self, env: &mut Environment) {
        let widget = (self.popup)(self.selected.clone(), self.focus.as_dyn(), self.enabled.as_dyn_read(), self.position.as_dyn_read(), self.dimension.as_dyn_read());

        env.transfer_widget(Some("controls_popup_layer".to_string()), WidgetTransferAction::Push(widget));
    }

    pub fn delegate(self, delegate: DelegateGenerator) -> PlainDatePicker<F, E> {
        Self::new_internal(
            self.selected,
            self.focus,
            self.enabled,
            delegate,
            self.popup,
        )
    }

    pub fn popup_delegate(self, popup_delegate: PopupDelegateGenerator) -> PlainDatePicker<F, E> {
        Self::new_internal(
            self.selected,
            self.focus,
            self.enabled,
            self.delegate,
            popup_delegate,
        )
    }

    fn new_internal<F2: State<T=Focus>, E2: ReadState<T=bool>>(
        selection: DateSelection,
        focus: F2,
        enabled: E2,
        //text_delegate: TextDelegateGenerator,
        delegate: DelegateGenerator,
        popup: PopupDelegateGenerator,
    ) -> PlainDatePicker<F2, E2> {
        let child = delegate(selection.clone(), focus.as_dyn(), enabled.as_dyn_read());

        PlainDatePicker {
            id: WidgetId::new(),
            position: LocalState::new(Position::new(0.0, 0.0)),
            dimension: LocalState::new(Dimension::new(100.0, 100.0)),
            focus,
            enabled,

            delegate,

            popup,
            child,

            selected: selection,
        }
    }
}


/*#[cfg(feature = "carbide_fluent")]
impl<F: State<T=Focus>, E: ReadState<T=bool>> PlainDatePicker<T, F, S, M, E> {
    pub fn localize(self) -> PlainDatePicker<T, F, S, M, E> {
        self.text_delegate(|item| carbide_fluent::LocalizedString::new(item).as_dyn_read())
    }
}*/

impl<F: State<T=Focus>, E: ReadState<T=bool>> CommonWidget for PlainDatePicker<F, E> {
    fn position(&self) -> Position {
        *self.position.value()
    }

    fn set_position(&mut self, position: Position) {
        self.position.set_value(position);
    }

    fn dimension(&self) -> Dimension {
        *self.dimension.value()
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension.set_value(dimension);
    }

    CommonWidgetImpl!(self, id: self.id, child: self.child, flag: WidgetFlag::FOCUSABLE, flexibility: 1, focus: self.focus);
}


impl<F: State<T=Focus>, E: ReadState<T=bool>> Debug for PlainDatePicker<F, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlainPopUpButton")
            .field("child", &self.child)
            .finish()
    }
}

impl<F: State<T=Focus>, E: ReadState<T=bool>> KeyboardEventHandler for PlainDatePicker<F, E> {
    fn handle_keyboard_event(&mut self, _event: &KeyboardEvent, _ctx: &mut KeyboardEventContext) {
        if self.get_focus() != Focus::Focused || !*self.enabled.value() { return; }

        /*if event == PopupButtonKeyCommand::Open {
            self.open_popup(ctx.env);
            //ctx.env.request_animation_frame();
        }*/
    }
}

impl<F: State<T=Focus>, E: ReadState<T=bool>> MouseEventHandler for PlainDatePicker<F, E> {
    // Implementing this instead of handle_mouse_event makes all the children not receive events.
    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        if !*ctx.is_current { return }
        match event {
            MouseEvent::Click(_, position, _) => {
                if self.is_inside(*position) {
                    if !*self.enabled.value() {
                        return;
                    }

                    if self.get_focus() != Focus::Focused {
                        self.set_focus(Focus::FocusRequested);
                        ctx.env.request_focus(Refocus::FocusRequest);
                    }
                    self.open_popup(ctx.env);
                    //ctx.env.request_animation_frame();
                } else {
                    if self.get_focus() == Focus::Focused {
                        self.set_focus(Focus::FocusReleased);
                        ctx.env.request_focus(Refocus::FocusRequest);
                    }
                }
            }
            _ => (),
        }
    }
}

impl<F: State<T=Focus>, E: ReadState<T=bool>> Update for PlainDatePicker<F, E> {
    fn update(&mut self, _ctx: &mut UpdateContext) {
        //self.popup.ensure_overlay_correct(ctx.env)
    }
}



// ---------------------------------------------------
//  Delegates
// ---------------------------------------------------
#[allow(unused)]
type TextDelegateGenerator = fn(DateSelection) ->Box<dyn AnyReadState<T=String>>;

type DelegateGenerator = fn(
    selected_item: DateSelection,
    focused: Box<dyn AnyState<T=Focus>>,
    enabled: Box<dyn AnyReadState<T=bool>>,
    //text_delegate: TextDelegateGenerator,
) -> Box<dyn AnyWidget>;

type PopupDelegateGenerator = fn(
    selected_item: DateSelection,
    focused: Box<dyn AnyState<T=Focus>>,
    enabled: Box<dyn AnyReadState<T=bool>>,
    parent_position: Box<dyn AnyReadState<T=Position>>,
    parent_dimension: Box<dyn AnyReadState<T=Dimension>>,
    //text_delegate: TextDelegateGenerator,
) -> Box<dyn AnyWidget>;

fn default_delegate(
    selection: DateSelection,
    focused: Box<dyn AnyState<T=Focus>>,
    _enabled: Box<dyn AnyReadState<T=bool>>,
    //text_delegate: TextDelegateGenerator,
) -> Box<dyn AnyWidget> {
    let background_color = Map1::read_map(focused.clone(), |focused| {
        match *focused {
            Focus::Focused => EnvironmentColor::Green,
            _ => EnvironmentColor::Blue,
        }
    });

    let text = match selection {
        DateSelection::Single(s) => {
            Map1::read_map(s, |s| {
                format!("{:?}", s)
            }).as_dyn_read()
        }
        DateSelection::Multi(s) => {
            Map1::read_map(s, |s| {
                format!("{:?}", s)
            }).as_dyn_read()
        }
        DateSelection::Range(s) => {
            Map1::read_map(s, |s| {
                format!("{:?}", s)
            }).as_dyn_read()
        }
    };

    ZStack::new((
        Rectangle::new().fill(background_color),
        Text::new(text),
    )).boxed()
}

fn default_popup_delegate(
    selection: DateSelection,
    _focused: Box<dyn AnyState<T=Focus>>,
    _enabled: Box<dyn AnyReadState<T=bool>>,
    _parent_position: Box<dyn AnyReadState<T=Position>>,
    _parent_dimension: Box<dyn AnyReadState<T=Dimension>>,
) -> Box<dyn AnyWidget> {
    PlainCalendar::new(selection)
        .padding(10.0)
        .background(Rectangle::new().fill(EnvironmentColor::SystemFill))
        .on_click(|_, _| {})
        .on_click_outside(|env, _| {
            env.transfer_widget(Some("controls_popup_layer".to_string()), WidgetTransferAction::Pop);
        })
        .boxed()
}