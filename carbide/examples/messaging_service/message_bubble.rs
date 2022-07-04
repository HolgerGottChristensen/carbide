use carbide::Widget;
use carbide_core::draw::{Dimension, Position};
use carbide_core::flags::Flags;
use carbide_core::prelude::TState;
use carbide_core::state::{LocalState, StateExt};
use carbide_core::widget::{
    CommonWidget, CornerRadii, EdgeInsets, HStack, IfElse, Rectangle, RoundedRectangle, Spacer,
    Text, Widget, WidgetExt, WidgetId, WidgetIter, WidgetIterMut, ZStack,
};
use carbide_core::CommonWidgetImpl;

#[derive(Debug, Clone)]
pub struct Message {
    text: String,
    sender: String,
}

impl Message {
    pub fn new(text: String, sender: String) -> Message {
        Message { text, sender }
    }
}

#[derive(Debug, Clone, Widget)]
pub struct MessageBubble {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    child: Box<dyn Widget>,
    external: TState<bool>,
}

impl MessageBubble {
    pub fn new(message: TState<Message>, me: String) -> Box<MessageBubble> {
        let message_state = message.mapped(|m: &Message| format!("{}: {}", m.sender, m.text));
        let is_me_state = message.mapped(move |m: &Message| *m.sender == me);
        let child = HStack::new(vec![
            IfElse::new(is_me_state.clone()).when_true(Spacer::new()),
            Text::new(message_state)
                .padding(10.0)
                .background(RoundedRectangle::new(6.0)),
            IfElse::new(is_me_state.clone()).when_false(Spacer::new()),
        ]);

        Box::new(MessageBubble {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child,
            external: LocalState::new(false),
        })
    }
}

CommonWidgetImpl!(MessageBubble, self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);

impl WidgetExt for MessageBubble {}
