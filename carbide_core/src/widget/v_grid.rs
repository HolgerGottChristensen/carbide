use smallvec::{SmallVec, smallvec};

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position, Scalar};
use crate::layout::{Layout, LayoutContext};
use crate::widget::{AnyWidget, CommonWidget, Widget, WidgetExt, WidgetId, WidgetSequence};

#[derive(Debug, Clone)]
pub enum VGridColumn {
    Fixed(f64),
    Adaptive(f64),
    Flexible {
        minimum: f64,
        maximum: f64,
    }
}

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct VGrid<W> where W: WidgetSequence {
    id: WidgetId,
    children: W,
    position: Position,
    dimension: Dimension,
    spacing: Dimension,
    columns: Vec<VGridColumn>,
    calculated_widths: Vec<f64>,
}

impl<W: WidgetSequence> VGrid<W> {
    pub fn new(children: W, columns: Vec<VGridColumn>) -> VGrid<W> {
        VGrid {
            id: WidgetId::new(),
            children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            spacing: Dimension::new(10.0, 10.0),
            columns,
            calculated_widths: vec![],
        }
    }

    pub fn spacing(mut self, spacing: Dimension) -> Self {
        self.spacing = spacing;
        self
    }
}

impl<W: WidgetSequence> Layout for VGrid<W> {

    // https://www.objc.io/blog/2020/11/23/grid-layout/
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.calculated_widths.clear();

        let total_width = requested_size.width;

        let mut remaining_width = total_width;

        // Subtract the spacings between the grid columns
        remaining_width = remaining_width - self.columns.len().saturating_sub(1) as f64 * self.spacing.width;

        // Subtract all the fixed columns
        remaining_width = remaining_width - self.columns.iter().filter_map(|col| match col {
            VGridColumn::Fixed(width) => Some(*width),
            VGridColumn::Adaptive(_) => None,
            VGridColumn::Flexible { .. } => None,
        }).sum::<f64>();

        let mut number_of_remaining_cols = self.columns.iter().filter(|a| !matches!(a, VGridColumn::Fixed(_))).count();

        // Iterate each remaining column in order
        for column in &self.columns {
            match column {
                VGridColumn::Fixed(w) => {
                    self.calculated_widths.push(*w);
                }
                VGridColumn::Adaptive(w) => {
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
                VGridColumn::Flexible { minimum, maximum } => {
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

        //

        let mut children_flexibility_rest: SmallVec<[(u32, &mut dyn AnyWidget); 25]> = smallvec![];

        self.children.foreach_mut(&mut |child| {
            children_flexibility_rest.push((child.flexibility(), child));
        });

        let mut remaining_rows = f64::ceil(children_flexibility_rest.len() as f64 / self.calculated_widths.len() as f64) as usize;

        let mut counter = 0;
        let mut remaining_height = requested_size.height - self.spacing.height * (remaining_rows - 1) as f64;
        let mut max_height = 0.0;
        let mut total_height = self.spacing.height * (remaining_rows - 1) as f64;

        remaining_rows += 1;

        for (_, widget) in children_flexibility_rest {
            if counter == 0 {
                remaining_height = (remaining_height - max_height).max(0.0);
                total_height += max_height;
                max_height = 0.0;
                remaining_rows -= 1;
            }

            let for_child = Dimension::new(
                self.calculated_widths[counter],
                max_height.max(remaining_height / remaining_rows as f64)
            );

            let actual_size = (*widget).calculate_size(for_child, ctx);

            max_height = max_height.max(actual_size.height);

            counter = (counter + 1) % self.calculated_widths.len();
        }

        self.dimension = Dimension::new(
            self.calculated_widths.iter().sum::<f64>() + (self.calculated_widths.len() - 1) as f64 * self.spacing.width,
            total_height + max_height,
        );

        self.dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let mut current_x = self.position.x;
        let mut current_y = self.position.y - self.spacing.height;
        let mut column_index = 0;
        let column_count = self.calculated_widths.len();
        let mut max_row_height = 0.0;

        self.children.foreach_mut(&mut |child| {
            if column_index == 0 {
                current_x = self.position.x;
                current_y += max_row_height + self.spacing.height;
                max_row_height = 0.0;
            }
            child.set_position(Position::new(
                current_x,
                current_y
            ));

            child.position_children(ctx);

            max_row_height = max_row_height.max(child.height());
            current_x += self.calculated_widths[column_index] + self.spacing.width;
            column_index = (column_index + 1) % column_count;
        })
    }
}

impl<W: WidgetSequence> CommonWidget for VGrid<W> {
    CommonWidgetImpl!(self, id: self.id, child: self.children, position: self.position, dimension: self.dimension, flexibility: 1);
}

impl<W: WidgetSequence> WidgetExt for VGrid<W> {}
