use std::fmt::Debug;

use carbide_core::{DeserializeOwned, Serialize};
use carbide_core::widget::*;

use crate::PlainButton;

#[derive(Clone, Widget)]
pub struct Button<T, GS> where T: 'static + Serialize + Clone + Debug + Default + DeserializeOwned, GS: GlobalState {
    id: Id,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    #[state] focus: FocusState<GS>,
    is_primary: bool,
    #[state] local_state: TState<T, GS>,
    on_click: fn(myself: &mut PlainButton<T, GS>, env: &mut Environment<GS>, global_state: &mut GS),
    display_item: Box<dyn Widget<GS>>,
}

impl<T: 'static + Serialize + Clone + Debug + Default + DeserializeOwned, GS: GlobalState> Button<T, GS> {
    pub fn new(display_item: Box<dyn Widget<GS>>) -> Box<Self> {
        let focus_state = CommonState::new_local_with_key(&Focus::Unfocused);

        let is_primary = true;

        let local_state = CommonState::new(&T::default());

        let clicked = |_: &mut PlainButton<T, GS>, _: &mut Environment<GS>, _: &mut GS| {};

        Self::new_internal(is_primary, focus_state.into(), display_item, local_state.into(), clicked)
    }

    pub fn on_click(self, fire: fn(myself: &mut PlainButton<T, GS>, env: &mut Environment<GS>, global_state: &mut GS)) -> Box<Self> {
        let focus_state = self.focus;
        let is_primary = self.is_primary;
        let local_state = self.local_state;
        let clicked = fire;
        let display_item = self.display_item;

        Self::new_internal(is_primary, focus_state, display_item, local_state, clicked)
    }

    pub fn local_state<K: Into<TState<T, GS>>>(self, state: K) -> Box<Self> {
        let focus_state = self.focus;
        let is_primary = self.is_primary;
        let local_state = state.into();
        let clicked = self.on_click;
        let display_item = self.display_item;

        Self::new_internal(is_primary, focus_state, display_item, local_state, clicked)
    }

    pub fn secondary(self) -> Box<Self> {
        let focus_state = self.focus;
        let is_primary = false;
        let local_state = self.local_state;
        let clicked = self.on_click;
        let display_item = self.display_item;

        Self::new_internal(is_primary, focus_state, display_item, local_state, clicked)
    }

    fn new_internal(is_primary: bool, focus_state: FocusState<GS>, display_item: Box<dyn Widget<GS>>, local_state: TState<T, GS>, clicked: fn(myself: &mut PlainButton<T, GS>, env: &mut Environment<GS>, global_state: &mut GS)) -> Box<Self> {
        let focus_color = TupleState3::new(
            focus_state.clone(),
            EnvironmentColor::OpaqueSeparator,
            EnvironmentColor::Accent,
        ).mapped(|(focus, primary_color, focus_color)| {
            if focus == &Focus::Focused {
                *focus_color
            } else {
                *primary_color
            }
        });

        let hover_state = CommonState::new_local_with_key(&false);
        let pressed_state = CommonState::new_local_with_key(&false);

        let normal_color = if is_primary {
            EnvironmentColor::Accent
        } else {
            EnvironmentColor::SecondarySystemBackground
        };

        let background_color = TupleState3::new(
            hover_state.clone(),
            pressed_state.clone(),
            normal_color,
        ).mapped(|(hover, pressed, normal_color)| {
            if *pressed {
                return normal_color.darkened(0.05)
            }
            if *hover {
                return normal_color.lightened(0.05)
            }

            *normal_color
        });

        let child = PlainButton::new(
            ZStack::initialize(vec![
                RoundedRectangle::initialize(CornerRadii::all(3.0))
                    .fill(background_color)
                    .stroke(focus_color)
                    .stroke_style(1.0),
                display_item.clone()
            ])
        ).local_state(local_state.clone())
            .focused(focus_state.clone())
            .on_click(clicked)
            .hover(hover_state)
            .pressed(pressed_state);

        Box::new(
            Button {
                id: Id::new_v4(),
                child,
                position: [0.0, 0.0],
                dimension: [235.0, 26.0],
                focus: focus_state,
                is_primary,
                local_state,
                on_click: clicked,
                display_item,
            }
        )
    }
}

impl<T: 'static + Serialize + Clone + Debug + Default + DeserializeOwned, GS: GlobalState> CommonWidget<GS> for Button<T, GS> {
    fn get_id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<GS> {
        WidgetIter::single(&self.child)
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<GS> {
        WidgetIterMut::single(&mut self.child)
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<GS> {
        WidgetIterMut::single(&mut self.child)
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<GS> {
        WidgetIterMut::single(&mut self.child)
    }

    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl<T: 'static + Serialize + Clone + Debug + Default + DeserializeOwned, GS: GlobalState> ChildRender for Button<T, GS> {}

impl<T: 'static + Serialize + Clone + Debug + Default + DeserializeOwned, GS: GlobalState> Layout<GS> for Button<T, GS> {
    fn flexibility(&self) -> u32 {
        5
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment<GS>) -> Dimensions {
        self.set_width(requested_size[0]);

        self.child.calculate_size(self.dimension, env);

        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.get_position();
        let dimension = self.get_dimension();


        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}


impl<T: 'static + Serialize + Clone + Debug + Default + DeserializeOwned, GS: GlobalState> WidgetExt<GS> for Button<T, GS> {}
