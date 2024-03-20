use smallvec::{SmallVec, smallvec};

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position, Scalar};
use crate::layout::{Layout, LayoutContext};
use crate::widget::{AnyWidget, CommonWidget, Widget, WidgetExt, WidgetId, WidgetSequence};

#[derive(Debug, Clone)]
pub enum HGridRow {
    Fixed(f64),
    Adaptive(f64),
    Flexible {
        minimum: f64,
        maximum: f64,
    }
}

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct HGrid<W> where W: WidgetSequence {
    id: WidgetId,
    children: W,
    position: Position,
    dimension: Dimension,
    spacing: Dimension,
    rows: Vec<HGridRow>,
    calculated_heights: Vec<f64>,
}

impl<W: WidgetSequence> HGrid<W> {
    pub fn new(children: W, columns: Vec<HGridRow>) -> HGrid<W> {
        HGrid {
            id: WidgetId::new(),
            children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            spacing: Dimension::new(10.0, 10.0),
            rows: columns,
            calculated_heights: vec![],
        }
    }

    pub fn spacing(mut self, spacing: Dimension) -> Self {
        self.spacing = spacing;
        self
    }
}

impl<W: WidgetSequence> Layout for HGrid<W> {

    // https://www.objc.io/blog/2020/11/23/grid-layout/
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.calculated_heights.clear();

        let total_height = requested_size.height;

        let mut remaining_height = total_height;

        // Subtract the spacings between the grid columns
        remaining_height = remaining_height - self.rows.len().saturating_sub(1) as f64 * self.spacing.height;

        // Subtract all the fixed columns
        remaining_height = remaining_height - self.rows.iter().filter_map(|row| match row {
            HGridRow::Fixed(height) => Some(*height),
            HGridRow::Adaptive(_) => None,
            HGridRow::Flexible { .. } => None,
        }).sum::<f64>();

        let mut number_of_remaining_rows = self.rows.iter().filter(|a| !matches!(a, HGridRow::Fixed(_))).count();

        // Iterate each remaining column in order
        for row in &self.rows {
            match row {
                HGridRow::Fixed(h) => {
                    self.calculated_heights.push(*h);
                }
                HGridRow::Adaptive(h) => {
                    let mut proposed_height = remaining_height / number_of_remaining_rows as Scalar;
                    remaining_height -= proposed_height;

                    let mut proposed_height2 = proposed_height;
                    let mut row_count = 1;
                    let mut spacing_count = 0;
                    proposed_height2 -= *h;

                    while proposed_height2 > self.spacing.height + *h {
                        proposed_height2 -= *h;
                        proposed_height2 -= self.spacing.height;
                        row_count += 1;
                        spacing_count += 1;
                    }

                    proposed_height -= spacing_count as f64 * self.spacing.height;

                    for _ in 0..row_count {
                        self.calculated_heights.push(proposed_height / row_count as f64);
                    }

                    number_of_remaining_rows -= 1;
                }
                HGridRow::Flexible { minimum, maximum } => {
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

        let mut children_flexibility_rest: SmallVec<[(u32, &mut dyn AnyWidget); 25]> = smallvec![];

        self.children.foreach_mut(&mut |child| {
            children_flexibility_rest.push((child.flexibility(), child));
        });

        let mut remaining_columns = f64::ceil(children_flexibility_rest.len() as f64 / self.calculated_heights.len() as f64) as usize;


        let mut counter = 0;
        let mut remaining_width = requested_size.width - self.spacing.width * (remaining_columns - 1) as f64;
        let mut max_width = 0.0;
        let mut total_width = self.spacing.width * (remaining_columns - 1) as f64;

        remaining_columns += 1;

        for (_, widget) in children_flexibility_rest {
            if counter == 0 {
                remaining_width = (remaining_width - max_width).max(0.0);
                total_width += max_width;
                max_width = 0.0;
                remaining_columns -= 1;
            }

            let for_child = Dimension::new(
                max_width.max(remaining_width / remaining_columns as f64),
                self.calculated_heights[counter]
            );

            let actual_size = (*widget).calculate_size(for_child, ctx);

            max_width = max_width.max(actual_size.width);

            counter = (counter + 1) % self.calculated_heights.len();
        }

        self.dimension = Dimension::new(
            total_width + max_width,
            self.calculated_heights.iter().sum::<f64>() + (self.calculated_heights.len() - 1) as f64 * self.spacing.height,
        );

        self.dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let mut current_y = self.position.y;
        let mut current_x = self.position.x - self.spacing.width;
        let mut row_index = 0;
        let row_count = self.calculated_heights.len();
        let mut max_column_width = 0.0;

        self.children.foreach_mut(&mut |child| {
            if row_index == 0 {
                current_y = self.position.y;
                current_x += max_column_width + self.spacing.width;
                max_column_width = 0.0;
            }

            child.set_position(Position::new(
                current_x,
                current_y
            ));

            child.position_children(ctx);

            max_column_width = max_column_width.max(child.width());
            current_y += self.calculated_heights[row_index] + self.spacing.height;
            row_index = (row_index + 1) % row_count;
        })
    }
}

impl<W: WidgetSequence> CommonWidget for HGrid<W> {
    CommonWidgetImpl!(self, id: self.id, child: self.children, position: self.position, dimension: self.dimension, flexibility: 1);
}

impl<W: WidgetSequence> WidgetExt for HGrid<W> {}
