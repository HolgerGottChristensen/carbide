use crate::draw::{Dimension, Position, Scalar};
use crate::layout::{Layout, LayoutContext};
use crate::widget::{AnyWidget, CommonWidget, CrossAxisAlignment, Sequence, Widget, WidgetId};
use crate::CommonWidgetImpl;
use smallvec::SmallVec;
use std::collections::HashMap;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct LazyHStack<W> where W: Sequence
{
    #[id] id: WidgetId,
    children: W,
    position: Position,
    dimension: Dimension,
    spacing: Scalar,
    cross_axis_alignment: CrossAxisAlignment,

    child_width_estimate: Option<Scalar>,
    child_widths: HashMap<WidgetId, Scalar>,

    current_indices: SmallVec<[usize; 32]>
}

impl<W: Sequence> LazyHStack<W> {
    pub fn new(children: W) -> LazyHStack<W> {
        LazyHStack {
            id: WidgetId::new(),
            children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            spacing: 0.0,
            cross_axis_alignment: CrossAxisAlignment::Center,
            child_width_estimate: None,
            child_widths: Default::default(),
            current_indices: Default::default(),
        }
    }

    pub fn cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Self {
        self.cross_axis_alignment = alignment;
        self
    }

    pub fn spacing(mut self, spacing: f64) -> Self {
        self.spacing = spacing;
        self
    }
}

impl<W: Sequence> Layout for LazyHStack<W> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let child_count = self.children.count();

        // If there are no children, we default to width 0.0
        if child_count == 0 {
            self.dimension = Dimension::new(0.0, requested_size.height);
        }

        let mut width_estimate = if let Some(width_estimate) = self.child_width_estimate {
            width_estimate
        } else {
            // Calculate width estimate based on the first N children
            let child = self.children.index(0);
            let chosen_size = child.calculate_size(requested_size, ctx);

            self.child_widths.insert(child.id(), chosen_size.width);

            self.child_width_estimate = Some(chosen_size.width);
            chosen_size.width
        };

        let offset = self.x();

        let mut cummulated_x = 0.0;


        self.current_indices.clear();
        // Determine which widget is expected at the current offset
        // This is a best guess, but can both be too low and too high.
        let estimate_for_index = width_estimate.max(1.0) + self.spacing;
        let mut index = (offset / estimate_for_index).floor().max(0.0) as usize;

        let estimated_start_x = index as Scalar * estimate_for_index;

        let bla = estimated_start_x - offset;

        loop {
            if index == child_count {
                break
            }

            self.current_indices.push(index);

            let child = self.children.index(index);

            // We set the relative offset of the child here. This is then offset in position_children.
            child.set_x(cummulated_x + estimated_start_x);

            let chosen_size = child.calculate_size(requested_size, ctx);

            let child_id = child.id();

            if !self.child_widths.contains_key(&child_id) {
                width_estimate = (width_estimate * self.child_widths.len() as Scalar + chosen_size.width) / (self.child_widths.len() + 1) as Scalar;
            }

            // TODO: Update height estimate when children are changing sizes dynamically
            self.child_widths.insert(child.id(), chosen_size.width);

            if cummulated_x + chosen_size.width > requested_size.width - bla - self.spacing {
                break
            }

            index += 1;
            cummulated_x += chosen_size.width + self.spacing;
        }

        self.child_width_estimate = Some(width_estimate);

        // We only estimate the height, and always use the requested width
        let total_width_estimate = width_estimate * child_count as Scalar
            + self.spacing * child_count.saturating_sub(1) as Scalar;

        self.dimension = Dimension::new(total_width_estimate, requested_size.height);

        self.dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let x = self.x();
        let y = self.y();
        let height = self.height();

        for current_index in &self.current_indices {
            let child = self.children.index(*current_index);
            child.set_x(child.x() + x);

            match self.cross_axis_alignment {
                CrossAxisAlignment::Start => child.set_y(y),
                CrossAxisAlignment::Center => child.set_y(height / 2.0 - child.height() / 2.0 + y),
                CrossAxisAlignment::End => child.set_y(y + height - child.height())
            }

            child.position_children(ctx);
        }
    }
}

impl<W: Sequence> CommonWidget for LazyHStack<W> {
    CommonWidgetImpl!(self, position: self.position, dimension: self.dimension, flexibility: 1);

    fn child(&mut self, index: usize) -> &mut dyn AnyWidget {
        self.children.index(index)
    }

    fn child_count(&mut self) -> usize {
        self.children.count()
    }

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        for current_index in &self.current_indices {
            let child = self.children.index(*current_index);
            f(child)
        }
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        todo!()
    }
}