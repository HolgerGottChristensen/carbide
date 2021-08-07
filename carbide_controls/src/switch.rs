use carbide_core::widget::*;

use crate::PlainSwitch;

#[derive(Clone, Widget)]
pub struct Switch<GS> where GS: GlobalStateContract {
    id: Id,
    child: PlainSwitch<GS>,
    position: Point,
    dimension: Dimensions,
}

impl<GS: GlobalStateContract> Switch<GS> {
    pub fn new<S: Into<StringState<GS>>, L: Into<BoolState<GS>>>(label: S, checked: L) -> Box<Self> {
        let mut child = *PlainSwitch::new(label, checked.into());

        child = *child.delegate(|focus_state, checked_state, button: Box<dyn Widget<GS>>| {
            let focus_color = TupleState3::new(
                focus_state,
                EnvironmentColor::OpaqueSeparator,
                EnvironmentColor::Accent,
            ).mapped(|(focus, primary_color, focus_color)| {
                if focus == &Focus::Focused {
                    *focus_color
                } else {
                    *primary_color
                }
            });

            let checked_color = TupleState3::new(
                checked_state.clone(),
                EnvironmentColor::SecondarySystemBackground,
                EnvironmentColor::Accent,
            ).mapped(|(selected, primary_color, checked_color)| {
                if *selected {
                    *checked_color
                } else {
                    *primary_color
                }
            });

            ZStack::initialize(vec![
                Capsule::initialize()
                    .fill(checked_color)
                    .stroke(focus_color)
                    .stroke_style(1.0),
                IfElse::new(checked_state)
                    .when_true(
                        HStack::initialize(vec![
                            Spacer::new(SpacerDirection::Horizontal),
                            Ellipse::new()
                                .fill(EnvironmentColor::DarkText)
                                .frame(22.0, 22.0),
                        ])
                    ).when_false(
                    HStack::initialize(vec![
                        Ellipse::new()
                            .fill(EnvironmentColor::DarkText)
                            .frame(22.0, 22.0),
                        Spacer::new(SpacerDirection::Horizontal),
                    ])
                ).padding(2.0),
                button,
            ]).frame(45.0, 26.0)
        });

        Box::new(Switch {
            id: Id::new_v4(),
            child,
            position: [0.0, 0.0],
            dimension: [235.0, 26.0],
        })
    }
}

impl<GS: GlobalStateContract> CommonWidget<GS> for Switch<GS> {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn children(&self) -> WidgetIter {
        WidgetIter::single(&self.child)
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
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

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl<GS: GlobalStateContract> ChildRender for Switch<GS> {}

impl<GS: GlobalStateContract> Layout<GS> for Switch<GS> {
    fn flexibility(&self) -> u32 {
        5
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment) -> Dimensions {
        self.set_width(requested_size[0]);

        self.child.calculate_size(self.dimension, env);

        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position();
        let dimension = self.dimension();


        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}


impl<GS: GlobalStateContract> WidgetExt<GS> for Switch<GS> {}
