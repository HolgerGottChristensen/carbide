use crate::draw::{Dimension, Position, Scalar};
use crate::layout::{Layout, LayoutContext};
use crate::widget::{AnyWidget, CommonWidget, GridItem, Sequence, Widget, WidgetId};
use crate::CommonWidgetImpl;
use smallvec::{SmallVec, ToSmallVec};
use std::collections::HashMap;
use carbide::draw::Rect;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct LazyVGrid<W> where W: Sequence
{
    #[id] id: WidgetId,
    children: W,
    position: Position,
    dimension: Dimension,
    spacing: Dimension,
    columns: SmallVec<[GridItem; 8]>,

    calculated_widths: SmallVec<[f64; 8]>,

    child_height_estimate: Option<Scalar>,
    child_heights: HashMap<WidgetId, Scalar>,
    requested_height: Scalar,

    current_indices: SmallVec<[usize; 32]>,
}

impl<W: Sequence> LazyVGrid<W> {
    pub fn new(columns: Vec<GridItem>, children: W) -> LazyVGrid<W> {
        LazyVGrid {
            id: WidgetId::new(),
            children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            spacing: Dimension::new(10.0, 10.0),
            columns: columns.to_smallvec(),
            calculated_widths: SmallVec::new(),
            child_height_estimate: None,
            child_heights: Default::default(),
            requested_height: 0.0,
            current_indices: Default::default(),
        }
    }

    pub fn spacing(mut self, spacing: Dimension) -> Self {
        self.spacing = spacing;
        self
    }
}

impl<W: Sequence> Layout for LazyVGrid<W> {

    // https://www.objc.io/blog/2020/11/23/grid-layout/
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.calculated_widths.clear();

        let child_count = self.children.count();

        // If there are no children, we default to height 0.0
        if child_count == 0 {
            self.current_indices.clear();
            self.dimension = Dimension::new(requested_size.width, 0.0);
        }

        let total_width = requested_size.width;

        let mut remaining_width = total_width;

        // Subtract the spacings between the grid columns
        remaining_width = remaining_width - self.columns.len().saturating_sub(1) as f64 * self.spacing.width;

        // Subtract all the fixed columns
        remaining_width = remaining_width - self.columns.iter().filter_map(|col| match col {
            GridItem::Fixed(width) => Some(*width),
            GridItem::Adaptive(_) => None,
            GridItem::Flexible => None,
            GridItem::MinMax { .. } => None,
        }).sum::<f64>();

        let mut number_of_remaining_cols = self.columns.iter().filter(|a| !matches!(a, GridItem::Fixed(_))).count();

        // Iterate each remaining column in order
        for column in &self.columns {
            match column {
                GridItem::Fixed(w) => {
                    self.calculated_widths.push(*w);
                }
                GridItem::Adaptive(w) => {
                    let mut proposed_width = remaining_width / number_of_remaining_cols as Scalar;
                    remaining_width -= proposed_width;

                    let mut proposed_width2 = proposed_width;
                    let mut column_count = 1;
                    let mut spacing_count = 0;
                    proposed_width2 -= *w;

                    while proposed_width2 > self.spacing.width + *w {
                        proposed_width2 -= *w;
                        proposed_width2 -= self.spacing.width;
                        column_count += 1;
                        spacing_count += 1;
                    }

                    proposed_width -= spacing_count as f64 * self.spacing.width;

                    for _ in 0..column_count {
                        self.calculated_widths.push(proposed_width / column_count as f64);
                    }

                    number_of_remaining_cols -= 1;
                }
                GridItem::Flexible => {
                    let proposed_width = remaining_width / number_of_remaining_cols as Scalar;
                    self.calculated_widths.push(proposed_width);
                    remaining_width -= proposed_width;
                    number_of_remaining_cols -= 1;
                }
                GridItem::MinMax { minimum, maximum } => {
                    let proposed_width = remaining_width / number_of_remaining_cols as Scalar;
                    if proposed_width < *minimum {
                        self.calculated_widths.push(*minimum);
                        remaining_width -= *minimum;
                    } else if proposed_width > *maximum {
                        self.calculated_widths.push(*maximum);
                        remaining_width -= *maximum;
                    } else {
                        self.calculated_widths.push(proposed_width);
                        remaining_width -= proposed_width;
                    }

                    number_of_remaining_cols -= 1;
                }
            }
        }

        let row_count = (child_count as Scalar / self.calculated_widths.len() as Scalar).ceil() as usize;

        if self.child_height_estimate.is_none() {
            // TODO: Calculate height estimate based on the a random selection of N children
            let child = self.children.index(0);
            let chosen_size = child.calculate_size(requested_size, ctx);

            self.child_heights.insert(child.id(), chosen_size.height);

            self.child_height_estimate = Some(chosen_size.height);
        }

        // We only estimate the height, and always use the requested width
        let total_height_estimate = self.child_height_estimate.unwrap() * row_count as Scalar
            + self.spacing.height * row_count.saturating_sub(1) as Scalar;

        self.dimension = Dimension::new(requested_size.width, total_height_estimate);
        self.requested_height = requested_size.height;

        self.dimension
    }

    fn position_children(&mut self, bounding_box: Rect, ctx: &mut LayoutContext) {
        self.current_indices.clear();

        let x = self.x();
        let y = self.y();
        let child_count = self.children.count();

        if child_count == 0 {
            return;
        }

        let mut height_estimate = self.child_height_estimate.expect("The child height can not be none after calculate_size");

        let offset = bounding_box.position.y - y;

        // Determine which widget is expected at the current offset
        // This is a best guess, but can both be too low and too high.
        let estimate_for_row = height_estimate.max(1.0) + self.spacing.height;
        let mut row = (offset / estimate_for_row).floor().max(0.0) as usize;

        let mut cummulated_y = row as Scalar * estimate_for_row + y;

        'outer: loop {
            let mut current_row_height: Scalar = 0.0;
            let mut cummulated_x = x;

            for row_offset in 0..self.calculated_widths.len() {
                let index = row * self.calculated_widths.len() + row_offset;

                if index == child_count {
                    break 'outer
                }

                self.current_indices.push(index);

                let child = self.children.index(index);

                let chosen_size = child.calculate_size(Dimension::new(
                    self.calculated_widths[row_offset],
                    self.requested_height
                ), ctx);

                // We set the relative offset of the child here. This is then offset in position_children.
                child.set_y(cummulated_y);
                child.set_x(cummulated_x);

                child.position_children(bounding_box, ctx);

                let child_id = child.id();

                if !self.child_heights.contains_key(&child_id) {
                    height_estimate = (height_estimate * self.child_heights.len() as Scalar + chosen_size.height) / (self.child_heights.len() + 1) as Scalar;
                }

                // TODO: Update height estimate when children are changing sizes dynamically
                self.child_heights.insert(child.id(), chosen_size.height);

                current_row_height = current_row_height.max(chosen_size.height);
                cummulated_x += self.calculated_widths[row_offset] + self.spacing.width;
            }

            if cummulated_y + current_row_height > bounding_box.top() {
                break
            }

            cummulated_y += current_row_height + self.spacing.height;
            row += 1;
        }

        self.child_height_estimate = Some(height_estimate);
    }
}

impl<W: Sequence> CommonWidget for LazyVGrid<W> {
    CommonWidgetImpl!(self, position: self.position, dimension: self.dimension, flexibility: 1);

    fn child(&mut self, index: usize) -> &mut dyn AnyWidget {
        self.children.index(index)
    }

    fn child_count(&mut self) -> usize {
        self.children.count()
    }

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        let count = self.children.count();
        for current_index in &self.current_indices {
            if *current_index < count {
                let child = self.children.index(*current_index);
                f(child)
            }
        }
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        todo!()
    }
}