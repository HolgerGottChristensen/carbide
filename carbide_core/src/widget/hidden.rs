use carbide::state::{IntoReadState, ReadState};
use carbide::widget::WidgetSync;
use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::render::{Render, RenderContext};
use crate::widget::{Empty, Widget, WidgetId, CommonWidget};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Hidden<W, S> where W: Widget, S: ReadState<T=bool> {
    #[id] id: WidgetId,
    child: W,
    position: Position,
    dimension: Dimension,
    #[state] state: S,
}

impl Hidden<Empty, bool> {
    pub fn new<W: Widget, S: IntoReadState<bool>>(child: W, state: S) -> Hidden<W, S::Output> {
        Hidden {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            state: state.into_read_state()
        }
    }

    pub fn new_unconditional<W: Widget>(child: W) -> Hidden<W, bool> {
        Hidden {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            state: true
        }
    }
}

impl<W: Widget, S: ReadState<T=bool>> CommonWidget for Hidden<W, S> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension);
}

impl<W: Widget, S: ReadState<T=bool>> Render for Hidden<W, S> {
    fn render(&mut self, ctx: &mut RenderContext) {
        self.sync(ctx.env);

        if !*self.state.value() {
            self.foreach_child(&mut |child| {
                child.render(ctx);
            });
        }
    }
}