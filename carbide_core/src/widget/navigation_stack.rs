use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::WidgetTransferAction;
use crate::lifecycle::{Update, UpdateContext};
use crate::widget::{AnyWidget, CommonWidget, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Update)]
pub struct NavigationStack {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    stack: Vec<Box<dyn AnyWidget>>,
    top: Box<dyn AnyWidget>,
    transfer_id: Option<String>,
}

impl NavigationStack {
    pub fn new(initial: Box<dyn AnyWidget>) -> NavigationStack {
        NavigationStack {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            stack: vec![],
            top: initial,
            transfer_id: None,
        }
    }

    pub fn transfer_id(mut self, transfer_id: impl Into<String>) -> Self {
        self.transfer_id = Some(transfer_id.into());
        self
    }
}

impl Update for NavigationStack {
    fn update(&mut self, ctx: &mut UpdateContext) {
        // Take out the transferred widget with the key if it exists
        if let Some(action) = ctx.env.transferred_widget(&self.transfer_id) {
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
                WidgetTransferAction::Replace(widget) => self.top = widget,
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

impl CommonWidget for NavigationStack {
    CommonWidgetImpl!(self, id: self.id, child: self.top, position: self.position, dimension: self.dimension);
}