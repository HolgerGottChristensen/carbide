use std::fmt::Debug;

use carbide_core::DeserializeOwned;
use carbide_core::draw::Dimension;
use carbide_core::Serialize;
use carbide_core::widget::*;

use crate::PlainPopUpButton;

#[derive(Clone, Widget)]
pub struct PopUpButton<T, GS> where GS: GlobalStateContract, T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static {
    id: Id,
    child: PlainPopUpButton<T, GS>,
    position: Point,
    dimension: Dimensions,
}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static, GS: GlobalStateContract> PopUpButton<T, GS> {
    pub fn new(model: Box<dyn State<Vec<T>, GS>>, selected_state: Box<dyn State<usize, GS>>) -> Box<Self> {
        let mut child = *PlainPopUpButton::new(model, selected_state);

        child = *child.display_item(|selected_item, focus_state| {
            let text = selected_item.mapped(|item| format!("{:?}", item));

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

            ZStack::initialize(vec![
                RoundedRectangle::new(CornerRadii::all(3.0))
                    .fill(EnvironmentColor::SecondarySystemBackground),
                HStack::new(vec![
                    Padding::init(EdgeInsets::single(0.0, 0.0, 7.0, 0.0), Text::new(text)),
                    Spacer::new(SpacerDirection::Horizontal),
                    ZStack::initialize(vec![
                        RoundedRectangle::new(CornerRadii::single(0.0, 0.0, 0.0, 2.0))
                            .fill(EnvironmentColor::Accent),
                        Canvas::initialize(|_, mut context| {
                            context.move_to(7.0, 10.0);
                            context.line_to(11.0, 6.0);
                            context.line_to(15.0, 10.0);
                            context.move_to(7.0, 14.0);
                            context.line_to(11.0, 18.0);
                            context.line_to(15.0, 14.0);
                            context.set_stroke_style(EnvironmentColor::DarkText);
                            context.set_line_width(1.5);
                            context.stroke();

                            context
                        }),
                    ]).padding(EdgeInsets::single(0.0, 0.0, 0.0, 1.0))
                        .frame(22.0, 24.0),
                ]),
                RoundedRectangle::new(CornerRadii::all(3.0))
                    .stroke_style(1.0)
                    .stroke(focus_color),
            ])
        });

        child = *child.display_item_popup(|item, _selected_index, _index, hovered| {
            let text = item.mapped(|item| format!("{:?}", item));

            let background_color = TupleState3::new(
                hovered.clone(),
                EnvironmentColor::Accent,
                EnvironmentColor::SecondarySystemBackground)
                .mapped(|(hovered, hover_color, other_color)| {
                    if *hovered {
                        *hover_color
                    } else {
                        *other_color
                    }
                });

            Rectangle::new(vec![
                HStack::new(vec![
                    Padding::init(
                        EdgeInsets::single(0.0, 0.0, 5.0, 0.0),
                        Text::new(text)
                            .color(EnvironmentColor::Label)),
                    Spacer::new(SpacerDirection::Horizontal),
                ])
            ]).fill(background_color)
                .stroke(EnvironmentColor::OpaqueSeparator)
                .stroke_style(1.0)
        });

        Box::new(PopUpButton {
            id: Id::new_v4(),
            child,
            position: [0.0, 0.0],
            dimension: [235.0, 26.0],
        })
    }
}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static, GS: GlobalStateContract> CommonWidget<GS> for PopUpButton<T, GS> {
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

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static, GS: GlobalStateContract> ChildRender for PopUpButton<T, GS> {}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static, GS: GlobalStateContract> Layout<GS> for PopUpButton<T, GS> {
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

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static, GS: GlobalStateContract> WidgetExt<GS> for PopUpButton<T, GS> {}
