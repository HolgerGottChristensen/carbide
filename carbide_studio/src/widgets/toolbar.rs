use carbide::CommonWidgetImpl;
use carbide::draw::{Alignment, Dimension, Position, Rect};
use carbide::environment::EnvironmentColor::Green;
use carbide::layout::{Layout, LayoutContext};
use carbide::render::{Render, RenderContext};
use carbide::widget::{AnySequence, AnyWidget, CommonWidget, HStack, Rectangle, Sequence, Widget, WidgetExt, WidgetId, WidgetSync};

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(Layout, Render)]
pub struct Toolbar {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    child: Box<dyn AnyWidget>,

    bar: Box<dyn AnyWidget>,
}

impl Toolbar {
    pub fn new<S: Sequence>(items: S, child: Box<dyn AnyWidget>) -> Toolbar {
        Toolbar {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child,
            bar: HStack::new(items)
                .spacing(5.0)
                .frame_fixed_height(26.0)
                .boxed(),
        }
    }
}

impl Layout for Toolbar {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let toolbar_height = 31.0;

        self.bar.calculate_size(Dimension::new(requested_size.width, toolbar_height), ctx);

        self.child.calculate_size(Dimension::new(requested_size.width, requested_size.height - toolbar_height), ctx);

        self.set_dimension(requested_size);
        requested_size
    }

    fn position_children(&mut self, bounding_box: Rect, ctx: &mut LayoutContext) {
        self.bar.set_position(self.position);
        self.bar.position_children(, ctx);

        self.child.set_position(Position::new(self.position.x, self.position.y + self.bar.height()));
        self.child.position_children(, ctx);
    }
}

impl Render for Toolbar {
    fn render(&mut self, ctx: &mut RenderContext) {
        self.bar.render(ctx);
        self.child.render(ctx);
    }
}

impl CommonWidget for Toolbar {
    CommonWidgetImpl!(self, child: [self.bar, self.child], position: self.position, dimension: self.dimension);
}