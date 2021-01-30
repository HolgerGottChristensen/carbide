use carbide_core::widget::*;
use carbide_core::color::RED;
use carbide_core::event_handler::KeyboardEvent;
use crate::plain::cursor::{Cursor, CursorIndex};
use carbide_core::state::environment::Environment;
use carbide_core::draw::shape::vertex::Vertex;
use carbide_core::widget::text::Wrap;


#[derive(Clone, Widget)]
#[event(handle_keyboard_event)]
pub struct PlainTextInput<GS> where GS: GlobalState {
    id: Id,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    #[state] text: State<String, GS>,
    cursor: Cursor,
    #[state] cursor_x: State<f64, GS>,
}

impl<GS: GlobalState> PlainTextInput<GS> {
    pub fn new() -> Box<Self> {

        let text_state = State::new_local_with_key(&String::from("Hello World!"));

        let cursor_x = State::new_local_with_key(&0.0);

        Box::new(PlainTextInput {
            id: Id::new_v4(),
            child: ZStack::initialize(vec![
                Text::initialize(text_state.clone())
                    .font_size(40.into()).wrap_mode(Wrap::None),
                Rectangle::initialize(vec![])
                    .fill(RED)
                    .frame(4.0, 40.0)
                    .offset(cursor_x.clone(), 0.0.into())
            ]).alignment(BasicLayouter::TopLeading),
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            text: text_state,
            cursor: Cursor::Single(CursorIndex{ line: 0, char: 0 }),
            cursor_x,
        })
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<GS>, global_state: &mut GS) {
        match event {
            KeyboardEvent::Text(string, _modifiers) => {
                self.text.get_value_mut(global_state).push_str(string);
                self.cursor = Cursor::Single(CursorIndex{ line: 0, char: self.text.get_value(global_state).len()-1 });
                println!("{:?}", self.cursor);

                let mut text_scaler: Box<carbide_core::widget::Text<GS>> = Text::initialize(self.text.get_value(global_state).clone().into())
                    .font_size(40.into()).wrap_mode(Wrap::None);

                text_scaler.set_position([0.0, 0.0]);
                text_scaler.set_dimension(self.dimension.add([100.0,100.0]));

                let positioned_glyphs = text_scaler.get_positioned_glyphs(env.get_fonts_map(), 1.0);

                println!("Number of positioned glyphs: {}", positioned_glyphs.len());

                let point = CursorIndex{ line: 0, char: self.text.get_value(global_state).len()-1 }.get_position(positioned_glyphs);

                println!("{:?}", point);

                *self.cursor_x.get_value_mut(global_state) = point[0];

            }
            _ => ()
        }
    }
}


impl<GS: GlobalState> CommonWidget<GS> for PlainTextInput<GS> {
    fn get_id(&self) -> Id {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<GS> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<GS> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<GS> {
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

impl<GS: GlobalState> ChildRender for PlainTextInput<GS> {}

impl<GS: GlobalState> SingleChildLayout for PlainTextInput<GS> {
    fn flexibility(&self) -> u32 {
        10
    }
}

impl<GS: GlobalState> WidgetExt<GS> for PlainTextInput<GS> {}