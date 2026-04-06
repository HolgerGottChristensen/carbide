use crate::draw::{Dimension, Position, Scalar};
use crate::layout::{Layout, LayoutContext};
use crate::widget::{AnyWidget, CommonWidget, GridItem, Sequence, Widget, WidgetId};
use crate::CommonWidgetImpl;
use smallvec::{SmallVec, ToSmallVec};
use std::collections::HashMap;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct LazyHGrid<W> where W: Sequence
{
    #[id] id: WidgetId,
    children: W,
    position: Position,
    dimension: Dimension,
    spacing: Dimension,
    rows: SmallVec<[GridItem; 8]>,

    calculated_heights: SmallVec<[f64; 8]>,

    child_width_estimate: Option<Scalar>,
    child_widths: HashMap<WidgetId, Scalar>,

    current_indices: SmallVec<[usize; 32]>
}

impl<W: Sequence> LazyHGrid<W> {
    pub fn new(columns: Vec<GridItem>, children: W) -> LazyHGrid<W> {
        LazyHGrid {
            id: WidgetId::new(),
            children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            spacing: Dimension::new(10.0, 10.0),
            rows: columns.to_smallvec(),
            calculated_heights: SmallVec::new(),
            child_width_estimate: None,
            child_widths: Default::default(),
            current_indices: Default::default(),
        }
    }

    pub fn spacing(mut self, spacing: Dimension) -> Self {
        self.spacing = spacing;
        self
    }
}

impl<W: Sequence> Layout for LazyHGrid<W> {

    // https://www.objc.io/blog/2020/11/23/grid-layout/
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.calculated_heights.clear();

        let child_count = self.children.count();

        // If there are no children, we default to height 0.0
        if child_count == 0 {
            self.current_indices.clear();
            self.dimension = Dimension::new(0.0, requested_size.height);
        }

        let total_heights = requested_size.height;

        let mut remaining_height = total_heights;

        // Subtract the spacings between the grid columns
        remaining_height = remaining_height - self.rows.len().saturating_sub(1) as f64 * self.spacing.height;

        // Subtract all the fixed columns
        remaining_height = remaining_height - self.rows.iter().filter_map(|row| match row {
            GridItem::Fixed(height) => Some(*height),
            GridItem::Adaptive(_) => None,
            GridItem::Flexible => None,
            GridItem::MinMax { .. } => None,
        }).sum::<f64>();

        let mut number_of_remaining_rows = self.rows.iter().filter(|a| !matches!(a, GridItem::Fixed(_))).count();

        // Iterate each remaining column in order
        for row in &self.rows {
            match row {
                GridItem::Fixed(w) => {
                    self.calculated_heights.push(*w);
                }
                GridItem::Adaptive(w) => {
                    let mut proposed_height = remaining_height / number_of_remaining_rows as Scalar;
                    remaining_height -= proposed_height;

                    let mut proposed_height2 = proposed_height;
                    let mut row_count = 1;
                    let mut spacing_count = 0;
                    proposed_height2 -= *w;

                    while proposed_height2 > self.spacing.height + *w {
                        proposed_height2 -= *w;
                        proposed_height2 -= self.spacing.height;
                        row_count += 1;
                        spacing_count += 1;
                    }

                    proposed_height -= spacing_count as f64 * self.spacing.width;

                    for _ in 0..row_count {
                        self.calculated_heights.push(proposed_height / row_count as f64);
                    }

                    number_of_remaining_rows -= 1;
                }
                GridItem::Flexible => {
                    let proposed_height = remaining_height / number_of_remaining_rows as Scalar;
                    self.calculated_heights.push(proposed_height);
                    remaining_height -= proposed_height;
                    number_of_remaining_rows -= 1;
                }
                GridItem::MinMax { minimum, maximum } => {
                    let proposed_height = remaining_height / number_of_remaining_rows as Scalar;
                    if proposed_height < *minimum {
                        self.calculated_heights.push(*minimum);
                        remaining_height -= *minimum;
                    } else if proposed_height > *maximum {
                        self.calculated_heights.push(*maximum);
                        remaining_height -= *maximum;
                    } else {
                        self.calculated_heights.push(proposed_height);
                        remaining_height -= proposed_height;
                    }

                    number_of_remaining_rows -= 1;
                }
            }
        }

        let column_count = (child_count as Scalar / self.calculated_heights.len() as Scalar).ceil() as usize;

        // Extract or calculate initial height estimate
        let mut width_estimate = if let Some(width_estimate) = self.child_width_estimate {
            width_estimate
        } else {
            // Calculate height estimate based on the first N children
            let child = self.children.index(0);
            let chosen_size = child.calculate_size(requested_size, ctx);

            self.child_widths.insert(child.id(), chosen_size.width);

            self.child_width_estimate = Some(chosen_size.width);
            chosen_size.width
        };

        let offset = -self.x();

        let mut cummulated_x = 0.0;

        self.current_indices.clear();

        let estimate_for_column = width_estimate.max(1.0) + self.spacing.width;
        let mut column = (offset / estimate_for_column).floor().max(0.0) as usize;

        let estimated_start_x = column as Scalar * estimate_for_column;

        let bla = estimated_start_x - offset;

        'outer: loop {
            let mut current_column_width: Scalar = 0.0;
            let mut cummulated_y = 0.0;

            for column_offset in 0..self.calculated_heights.len() {
                let index = column * self.calculated_heights.len() + column_offset;

                if index == child_count {
                    break 'outer
                }

                self.current_indices.push(index);

                let child = self.children.index(index);

                // We set the relative offset of the child here. This is then offset in position_children.
                child.set_y(cummulated_y);
                child.set_x(cummulated_x + estimated_start_x);

                let for_child = Dimension::new(
                    requested_size.width,
                    self.calculated_heights[column_offset],
                );

                let chosen_size = child.calculate_size(for_child, ctx);

                let child_id = child.id();

                if !self.child_widths.contains_key(&child_id) {
                    width_estimate = (width_estimate * self.child_widths.len() as Scalar + chosen_size.width) / (self.child_widths.len() + 1) as Scalar;
                }

                // TODO: Update height estimate when children are changing sizes dynamically
                self.child_widths.insert(child.id(), chosen_size.width);

                current_column_width = current_column_width.max(chosen_size.width);
                cummulated_y += self.calculated_heights[column_offset] + self.spacing.height;
            }

            if cummulated_x + current_column_width > requested_size.width - bla - self.spacing.width {
                break
            }

            cummulated_x += current_column_width + self.spacing.width;
            column += 1;
        }

        self.child_width_estimate = Some(width_estimate);

        // We only estimate the height, and always use the requested width
        let total_width_estimate = width_estimate * column_count as Scalar
            + self.spacing.width * column_count.saturating_sub(1) as Scalar;

        self.dimension = Dimension::new(total_width_estimate, requested_size.height);

        self.dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let x = self.x();
        let y = self.y();

        for current_index in &self.current_indices {
            let child = self.children.index(*current_index);
            child.set_y(child.y() + y);
            child.set_x(child.x() + x);

            child.position_children(ctx);
        }
    }
}

impl<W: Sequence> CommonWidget for LazyHGrid<W> {
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