use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, WidgetTransferAction};
use carbide_core::render::{Primitive, Render};
use carbide_core::Widget;
use carbide_core::widget::{Id, Widget, WidgetExt};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct NavigationStack {
    id: Id,
    position: Position,
    dimension: Dimension,
    stack: Vec<Box<dyn Widget>>,
    top: Box<dyn Widget>,
    transfer_id: Option<String>,
}

impl NavigationStack {
    pub fn new(initial: Box<dyn Widget>) -> Box<NavigationStack> {
        Box::new(NavigationStack {
            id: Id::new_v4(),
            position: Default::default(),
            dimension: Default::default(),
            stack: vec![],
            top: initial,
            transfer_id: None,
        })
    }

    pub fn transfer_id(mut self, transfer_id: impl Into<String>) -> Box<Self> {
        self.transfer_id = Some(transfer_id.into());
        Box::new(self)
    }
}

impl Render for NavigationStack {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        // Draw first because we are sure it is laid out.
        self.top.process_get_primitives(primitives, env);

        // Take out the transferred widget with the key if it exists
        if let Some(action) = env.transferred_widget(self.transfer_id.clone()) {
            match action {
                WidgetTransferAction::Push(mut widget) => {
                    let top = &mut self.top;
                    std::mem::swap(top, &mut widget);
                    self.stack.push(widget)
                }
                WidgetTransferAction::Pop => {
                    if let Some(new_top) = self.stack.pop() {
                        self.top = new_top
                    }
                }
                WidgetTransferAction::Replace(widget) => {
                    self.top = widget
                }
                WidgetTransferAction::PushVec(vec) => {
                    for mut widget in vec {
                        let top = &mut self.top;
                        std::mem::swap(top, &mut widget);
                        self.stack.push(widget)
                    }
                }
                WidgetTransferAction::PopN(n) => {
                    for _ in 0..n {
                        if let Some(new_top) = self.stack.pop() {
                            self.top = new_top
                        }
                    }
                }
                WidgetTransferAction::PopAll => {
                    if self.stack.len() > 0 {
                        self.top = self.stack.remove(0);
                        self.stack = vec![];
                    }
                }
                WidgetTransferAction::ReplaceAll(widget) => {
                    self.stack = vec![];
                    self.top = widget
                }
            }
        }
    }
}

CommonWidgetImpl!(NavigationStack, self, id: self.id, child: self.top, position: self.position, dimension: self.dimension);

impl WidgetExt for NavigationStack {}