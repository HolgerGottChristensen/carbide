use carbide_core::draw::{Dimension, Position};
use carbide_core::flags::Flags;
use carbide_core::prelude::TState;
use carbide_core::widget::{CommonWidget, CornerRadii, EdgeInsets, HStack, Id, IfElse, Rectangle, RoundedRectangle, Spacer, Text, Widget, WidgetExt, WidgetIter, WidgetIterMut, ZStack};
use carbide::Widget;
use carbide_core::state::{LocalState, StateExt};

#[derive(Debug, Clone)]
pub struct Message {
    text: String,
    sender: String,
}

impl Message {
    pub fn new(text: String, sender: String) -> Message {
        Message {
            text,
            sender,
        }
    }
}

#[derive(Debug, Clone, Widget)]
pub struct MessageBubble {
    id: Id,
    position: Position,
    dimension: Dimension,
    child: Box<dyn Widget>,
    external: TState<bool>,
}

impl MessageBubble {
    pub fn new(message: TState<Message>, me: String) -> Box<MessageBubble> {
        let message_state = message.mapped(|m: &Message| {
            format!("{}: {}", m.sender, m.text)
        });
        let is_me_state = message.mapped(move |m: &Message| {
            *m.sender == me
        });
        let child = HStack::new(vec![
            IfElse::new(is_me_state.clone())
                .when_true(Spacer::new()),
            Rectangle::new(vec![
                Text::new(message_state)
                    .padding(20.0),
            ]).shrink_to_fit(),
            IfElse::new(is_me_state.clone())
                .when_false(Spacer::new()),
        ]);

        Box::new(MessageBubble {
            id: Id::new_v4(),
            position: Default::default(),
            dimension: Default::default(),
            child,
            external: LocalState::new(false),
        })
    }
}

impl CommonWidget for MessageBubble {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
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

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl WidgetExt for MessageBubble {}