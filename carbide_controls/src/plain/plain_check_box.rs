use carbide_core::draw::Dimension;
use carbide_core::prelude::Uuid;
use carbide_core::state::state::State;
use carbide_core::widget::*;

use crate::PlainButton;
use crate::types::*;

#[derive(Clone, Widget)]
#[focusable(block_focus)]
pub struct PlainCheckBox<GS> where GS: GlobalStateContract {
    id: Id,
    #[state] focus: FocusState<GS>,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    delegate: fn(focus: FocusState<GS>, checked: CheckBoxState<GS>, button: Box<dyn Widget<GS>>) -> Box<dyn Widget<GS>>,
    label: StringState<GS>,
    #[state] checked: CheckBoxState<GS>,
}

impl<GS: GlobalStateContract> PlainCheckBox<GS> {
    pub fn focused<K: Into<FocusState<GS>>>(mut self, focused: K) -> Box<Self> {
        self.focus = focused.into();
        Box::new(self)
    }

    pub fn new<S: Into<StringState<GS>>, L: Into<CheckBoxState<GS>>>(label: S, checked: L) -> Box<Self> {
        let focus_state = Box::new(CommonState::new_local_with_key(&Focus::Unfocused));

        let default_delegate = |_focus_state: FocusState<GS>, checked: CheckBoxState<GS>, button: Box<dyn Widget<GS>>| -> Box<dyn Widget<GS>> {
            let highlight_color = TupleState4::new(checked, EnvironmentColor::Red, EnvironmentColor::Green, EnvironmentColor::Blue)
                .mapped(|(selected, true_color, intermediate_color, false_color)| {
                    match *selected {
                        CheckBoxValue::True => {
                            *true_color
                        }
                        CheckBoxValue::Intermediate => {
                            *intermediate_color
                        }
                        CheckBoxValue::False => {
                            *false_color
                        }
                    }
                });

            Rectangle::new(vec![
                button
            ]).fill(highlight_color)
        };

        Self::new_internal(checked.into(), focus_state.into(), default_delegate, label.into())
    }

    pub fn delegate(self, delegate: fn(focus: FocusState<GS>, selected: CheckBoxState<GS>, button: Box<dyn Widget<GS>>) -> Box<dyn Widget<GS>>) -> Box<Self> {
        let checked = self.checked;
        let focus_state = self.focus;
        let label_state = self.label;

        Self::new_internal(checked, focus_state, delegate, label_state)
    }

    fn new_internal(
        checked: CheckBoxState<GS>,
        focus_state: FocusState<GS>,
        delegate: fn(focus: FocusState<GS>, selected: CheckBoxState<GS>, button: Box<dyn Widget<GS>>) -> Box<dyn Widget<GS>>,
        label_state: StringState<GS>,
    ) -> Box<Self> {
        let button = PlainButton::<CheckBoxValue, GS>::new(Spacer::new(SpacerDirection::Vertical))
            .local_state(checked.clone())
            .on_click(|myself, env, global_state| {
                let checked = myself.get_local_state().get_value_mut(env, global_state);

                if *checked == CheckBoxValue::True {
                    *checked = CheckBoxValue::False
                } else {
                    *checked = CheckBoxValue::True;
                }

                myself.set_focus_and_request(Focus::FocusRequested, env);
            }).focused(focus_state.clone());

        let delegate_widget = delegate(focus_state.clone(), checked.clone(), button);

        let child = HStack::new(vec![
            delegate_widget,
            Text::new(label_state.clone()),
            Spacer::new(SpacerDirection::Horizontal),
        ]).spacing(5.0);

        Box::new(PlainCheckBox {
            id: Id::new_v4(),
            focus: focus_state,
            child,
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            delegate,
            label: label_state,
            checked,
        })
    }
}

impl<GS: GlobalStateContract> CommonWidget<GS> for PlainCheckBox<GS> {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn flag(&self) -> Flags {
        Flags::FOCUSABLE
    }

    fn children(&self) -> WidgetIter {
        if self.child.flag() == Flags::PROXY {
            self.child.children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        if self.child.flag() == Flags::PROXY {
            self.child.children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn proxied_children(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn proxied_children_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl<GS: GlobalStateContract> ChildRender for PlainCheckBox<GS> {}

impl<GS: GlobalStateContract> Layout<GS> for PlainCheckBox<GS> {
    fn flexibility(&self) -> u32 {
        10
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment) -> Dimensions {
        if let Some(child) = self.children_mut().next() {
            child.calculate_size(requested_size, env);
        }

        self.set_dimension(requested_size);

        requested_size
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position();
        let dimension = self.dimension();

        if let Some(child) = self.children_mut().next() {
            positioning(position, dimension, child);
            child.position_children();
        }
    }
}


impl<GS: GlobalStateContract> WidgetExt<GS> for PlainCheckBox<GS> {}
