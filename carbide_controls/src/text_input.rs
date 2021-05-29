use carbide_core::widget::*;

use crate::PlainTextInput;

#[derive(Clone, Widget)]
pub struct TextInput<GS> where GS: GlobalState {
    id: Id,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    #[state] focus: FocusState<GS>,
}

impl<GS: GlobalState> TextInput<GS> {
    pub fn new<S: Into<StringState<GS>>>(text: S) -> Box<Self> {
        let text = text.into();

        let focus_state = CommonState::new_local_with_key(&Focus::Unfocused);

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

        let child = ZStack::initialize(vec![
            RoundedRectangle::initialize(CornerRadii::all(3.0))
                .fill(EnvironmentColor::SecondarySystemBackground)
                .stroke(focus_color)
                .stroke_style(1.0),
            PlainTextInput::new(text)
                .font_size(EnvironmentFontSize::Body)
                .focus_state(focus_state.clone())
                .clip()
                .padding(EdgeInsets::single(0.0, 0.0, 5.0, 5.0))
        ]);


        Box::new(
            TextInput {
                id: Id::new_v4(),
                child,
                position: [0.0, 0.0],
                dimension: [235.0, 26.0],
                focus: focus_state.into(),
            }
        )
    }
}

impl<GS: GlobalState> CommonWidget<GS> for TextInput<GS> {
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

impl<GS: GlobalState> ChildRender for TextInput<GS> {}

impl<GS: GlobalState> Layout<GS> for TextInput<GS> {
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


impl<GS: GlobalState> WidgetExt<GS> for TextInput<GS> {}
