use crate::draw::{Dimension, Position, Scalar};
use crate::layout::{Layout, LayoutContext};
use crate::widget::{AnyWidget, CommonWidget, CrossAxisAlignment, Sequence, Widget, WidgetId};
use crate::CommonWidgetImpl;
use smallvec::SmallVec;
use std::collections::HashMap;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct LazyVStack<W> where W: Sequence
{
    #[id] id: WidgetId,
    children: W,
    position: Position,
    dimension: Dimension,
    spacing: Scalar,
    cross_axis_alignment: CrossAxisAlignment,

    child_height_estimate: Option<Scalar>,
    child_heights: HashMap<WidgetId, Scalar>,

    current_indices: SmallVec<[usize; 32]>
}

impl<W: Sequence> LazyVStack<W> {
    pub fn new(children: W) -> LazyVStack<W> {
        LazyVStack {
            id: WidgetId::new(),
            children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            spacing: 0.0,
            cross_axis_alignment: CrossAxisAlignment::Center,
            child_height_estimate: None,
            child_heights: Default::default(),
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

impl<W: Sequence> Layout for LazyVStack<W> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let child_count = self.children.count();

        // If there are no children, we default to height 0.0
        if child_count == 0 {
            self.current_indices.clear();
            self.dimension = Dimension::new(requested_size.width, 0.0);
        }

        let mut height_estimate = if let Some(height_estimate) = self.child_height_estimate {
            height_estimate
        } else {
            // Calculate height estimate based on the first N children
            let child = self.children.index(0);
            let chosen_size = child.calculate_size(requested_size, ctx);

            self.child_heights.insert(child.id(), chosen_size.height);

            self.child_height_estimate = Some(chosen_size.height);
            chosen_size.height
        };

        let offset = -self.y();

        let mut cummulated_y = 0.0;


        self.current_indices.clear();
        // Determine which widget is expected at the current offset
        // This is a best guess, but can both be too low and too high.
        let estimate_for_index = height_estimate.max(1.0) + self.spacing;
        let mut index = (offset / estimate_for_index).floor().max(0.0) as usize;

        let estimated_start_y = index as Scalar * estimate_for_index;

        let bla = estimated_start_y - offset;

        loop {
            if index == child_count {
                break
            }

            self.current_indices.push(index);

            let child = self.children.index(index);

            // We set the relative offset of the child here. This is then offset in position_children.
            child.set_y(cummulated_y + estimated_start_y);

            let chosen_size = child.calculate_size(requested_size, ctx);

            let child_id = child.id();

            if !self.child_heights.contains_key(&child_id) {
                height_estimate = (height_estimate * self.child_heights.len() as Scalar + chosen_size.height) / (self.child_heights.len() + 1) as Scalar;
            }

            // TODO: Update height estimate when children are changing sizes dynamically
            self.child_heights.insert(child.id(), chosen_size.height);

            if cummulated_y + chosen_size.height > requested_size.height - bla - self.spacing {
                break
            }

            index += 1;
            cummulated_y += chosen_size.height + self.spacing;
        }

        self.child_height_estimate = Some(height_estimate);

        // We only estimate the height, and always use the requested width
        let total_height_estimate = height_estimate * child_count as Scalar
            + self.spacing * child_count.saturating_sub(1) as Scalar;

        self.dimension = Dimension::new(requested_size.width, total_height_estimate);

        self.dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let x = self.x();
        let y = self.y();
        let width = self.width();

        for current_index in &self.current_indices {
            let child = self.children.index(*current_index);
            child.set_y(child.y() + y);

            match self.cross_axis_alignment {
                CrossAxisAlignment::Start => child.set_x(x),
                CrossAxisAlignment::Center => child.set_x(width / 2.0 - child.width() / 2.0 + x),
                CrossAxisAlignment::End => child.set_x(x + width - child.width())
            }

            child.position_children(ctx);
        }
    }
}

impl<W: Sequence> CommonWidget for LazyVStack<W> {
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