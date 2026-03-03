use crate::size_collection::SizeCollection;
use crate::style::{SpreadsheetStyle, TableStyle};
use carbide::draw::fill::FillOptions;
use carbide::draw::stroke::StrokeOptions;
use carbide::draw::{DrawOptions, DrawShape, Rect, Scalar};
use carbide::math::{Matrix4, Vector3};
use carbide::state::{LocalState, ReadState, State};
use carbide_core::draw::{Dimension, Position};
use carbide_core::event::{MouseEvent, MouseEventContext, MouseEventHandler};
use carbide_core::layout::{Layout, LayoutContext};
use carbide_core::render::{Render, RenderContext};
use carbide_core::widget::{CommonWidget, Widget, WidgetExt, WidgetId};
use carbide_core::CommonWidgetImpl;
use std::fmt::Debug;

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(MouseEvent, Render, Layout)]
pub struct Table<S> where S: TableStyle + Clone {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,
    offset: LocalState<Position>,

    style: S,

    widths: Vec<f64>,
    heights: Vec<f64>,
}

impl Table<SpreadsheetStyle> {
    pub fn new() -> Table<SpreadsheetStyle> {
        Table {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            offset: LocalState::new(Position::origin()),
            widths: vec![100.0; 40],
            heights: vec![21.0; 100],

            style: SpreadsheetStyle {
                frozen_columns: LocalState::new(1),
                frozen_rows: LocalState::new(1),
            }
        }
    }
}

impl<S: TableStyle + Clone> MouseEventHandler for Table<S> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, _ctx: &mut MouseEventContext) {
        match event {
            MouseEvent::Scroll { x, y, mouse_position, .. } => {
                if !self.is_inside(*mouse_position) {
                    return;
                }

                let mut new_x = self.offset.value().x;
                let mut new_y = self.offset.value().y;

                new_x -= x;
                new_y += y;

                new_x = new_x.clamp(0.0, (self.widths.iter().sum::<f64>() - self.dimension.width).max(0.0));
                new_y = new_y.clamp(0.0, (self.heights.iter().sum::<f64>() - self.dimension.height).max(0.0));

                self.offset.value_mut().y = new_y;
                self.offset.value_mut().x = new_x;
            }
            _ => ()
        }
    }
}

impl<S: TableStyle + Clone> Layout for Table<S> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.set_dimension(requested_size);
        requested_size
    }
}

impl<S: TableStyle + Clone> Render for Table<S> {
    fn render(&mut self, ctx: &mut RenderContext) {
        let offset_x = self.offset.value().x;
        let offset_y = self.offset.value().y;

        let scroll_x = self.position.x - offset_x;
        let scroll_y = self.position.y - offset_y;

        let frozen_columns = self.style.frozen_columns();
        let frozen_rows = self.style.frozen_rows();

        let has_frozen_columns = frozen_columns > 0;
        let has_frozen_rows = frozen_rows > 0;

        let frozen_width = self.widths[0..frozen_columns].iter().sum::<Scalar>();
        let frozen_height = self.heights[0..frozen_rows].iter().sum::<Scalar>();

        let scrollable_start_x = offset_x + frozen_width;
        let scrollable_start_y = offset_y + frozen_height;

        let scrollable_end_y = scrollable_start_y + self.dimension.height - frozen_height;
        let scrollable_end_x = scrollable_start_x + self.dimension.width - frozen_width;

        // The transform handles the scrolling of the grid
        ctx.transform(Matrix4::from_translation(Vector3::new(scroll_x as f32, scroll_y as f32, 0.0)), |ctx| {
            // Draw the backgrounds for each visible moving cell
            for (cumulative_height, height, row) in self.heights.iter_range(scrollable_start_y, scrollable_end_y) {
                for (cumulative_width, width, col) in self.widths.iter_range(scrollable_start_x, scrollable_end_x) {

                    // We only draw moving cells, so we skip the frozen rows and columns
                    if row < frozen_rows as u32 || col < frozen_columns as u32 {
                        continue;
                    }

                    let cell = self.style.cell(col, row);

                    ctx.style(cell.draw_style, |ctx| {
                        ctx.shape(
                            DrawShape::Rectangle(Rect::new(
                                Position::new(cumulative_width, cumulative_height),
                                Dimension::new(width, height)
                            )),
                            DrawOptions::Fill(FillOptions::default()),
                        );
                    });



                    if let Some(text) = &cell.text {
                        let text_dimensions = ctx.measure_text(text, &cell.text_style, None);

                        let position = Position::new(
                            cumulative_width + width / 2.0 - text_dimensions.width / 2.0,
                            cumulative_height + height / 2.0 - text_dimensions.height / 2.0
                        );

                        ctx.text(text, &cell.text_style, position, None);
                    }
                }
            }

            let mut first_column = true;
            let mut first_row = true;

            // Draw vertical and horizontal lines for moving visible cells
            for (cummulative_height, height, row) in self.heights.iter_range(scrollable_start_y, scrollable_end_y) {
                for (cummulative_width, width, col) in self.widths.iter_range(scrollable_start_x, scrollable_end_x) {

                    if row < frozen_rows as u32 || col < frozen_columns as u32 {
                        continue;
                    }

                    let cell_borders = self.style.cell_border(row, col);

                    ctx.style(cell_borders.left.style, |ctx| {
                        if first_column && !has_frozen_columns {
                            first_column = false;

                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(cummulative_width, cummulative_height),
                                    Position::new(cummulative_width, cummulative_height + height)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.left.width)),
                            );
                        }
                    });

                    ctx.style(cell_borders.right.style, |ctx| {
                        ctx.shape(
                            DrawShape::Line(
                                Position::new(cummulative_width + width, cummulative_height),
                                Position::new(cummulative_width + width, cummulative_height + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.right.width)),
                        );
                    });

                    ctx.style(cell_borders.top.style, |ctx| {
                        if first_row && !has_frozen_rows {
                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(cummulative_width, cummulative_height),
                                    Position::new(cummulative_width + width, cummulative_height)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.top.width)),
                            );
                        }
                    });

                    ctx.style(cell_borders.bottom.style, |ctx| {
                        ctx.shape(
                            DrawShape::Line(
                                Position::new(cummulative_width, cummulative_height + height),
                                Position::new(cummulative_width + width, cummulative_height + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.bottom.width)),
                        );
                    });
                }

                first_column = true;
                first_row = false;
            }

            first_row = true;
            first_column = true;

            let mut frozen_cummulative_width = 0.0;
            let mut frozen_cummulative_height = 0.0;

            // Frozen rows
            for (row, height) in self.heights[0..frozen_rows].iter().enumerate() {
                for (cummulative_width, width, col) in self.widths.iter_range(scrollable_start_x, scrollable_start_x + self.dimension.width - frozen_width) {

                    if col < frozen_columns as u32 {
                        continue;
                    }

                    let position = Position::new(cummulative_width, frozen_cummulative_height + offset_y);

                    let cell = self.style.cell(col, row as u32);

                    ctx.style(cell.draw_style, |ctx| {
                        ctx.shape(
                            DrawShape::Rectangle(Rect::new(position, Dimension::new(width, *height))),
                            DrawOptions::Fill(FillOptions::default()),
                        );
                    });

                    if let Some(text) = &cell.text {
                        ctx.text(text, &cell.text_style, Position::new(cummulative_width + 3.0, frozen_cummulative_height + offset_y + 4.0), None);
                    }
                }

                frozen_cummulative_height += *height;
            }

            frozen_cummulative_width = 0.0;
            frozen_cummulative_height = 0.0;
            first_row = true;
            first_column = true;

            for (row, height) in self.heights[0..frozen_rows].iter().enumerate() {
                for (cummulative_width, width, col) in self.widths.iter_range(scrollable_start_x, scrollable_start_x + self.dimension.width - frozen_width) {

                    if col < frozen_columns as u32 {
                        continue;
                    }

                    let cell_borders = self.style.cell_border(row as u32, col);

                    ctx.style(cell_borders.left.style, |ctx| {
                        if first_column && !has_frozen_columns {
                            first_column = false;

                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(cummulative_width, frozen_cummulative_height + offset_y),
                                    Position::new(cummulative_width, frozen_cummulative_height + offset_y + height)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.left.width)),
                            );
                        }
                    });

                    ctx.style(cell_borders.right.style, |ctx| {
                        ctx.shape(
                            DrawShape::Line(
                                Position::new(cummulative_width + width, frozen_cummulative_height + offset_y),
                                Position::new(cummulative_width + width, frozen_cummulative_height + offset_y + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.right.width)),
                        );
                    });

                    ctx.style(cell_borders.top.style, |ctx| {
                        if first_row {
                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(cummulative_width, frozen_cummulative_height + offset_y),
                                    Position::new(cummulative_width + width, frozen_cummulative_height + offset_y)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.top.width)),
                            );
                        }
                    });

                    ctx.style(cell_borders.bottom.style, |ctx| {
                        ctx.shape(
                            DrawShape::Line(
                                Position::new(cummulative_width, frozen_cummulative_height + offset_y + height),
                                Position::new(cummulative_width + width, frozen_cummulative_height + offset_y + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.bottom.width)),
                        );
                    });
                }

                first_column = true;
                first_row = false;
                frozen_cummulative_height += *height;
            }

            // Frozen columns
            frozen_cummulative_width = 0.0;
            frozen_cummulative_height = 0.0;

            for (col, width) in self.widths[0..frozen_columns].iter().enumerate() {
                for (cummulative_height, height, row) in self.heights.iter_range(scrollable_start_y, scrollable_start_y + self.dimension.height - frozen_height) {

                    if row < frozen_rows as u32 {
                        continue;
                    }

                    let position = Position::new(frozen_cummulative_width + offset_x, cummulative_height);

                    let cell = self.style.cell(col as u32, row);

                    ctx.style(cell.draw_style, |ctx| {
                        ctx.shape(
                            DrawShape::Rectangle(Rect::new(position, Dimension::new(*width, height))),
                            DrawOptions::Fill(FillOptions::default()),
                        );
                    });

                    if let Some(text) = &cell.text {
                        ctx.text(text, &cell.text_style, Position::new(frozen_cummulative_width + offset_x + 3.0, cummulative_height + 4.0), None);
                    }

                }

                frozen_cummulative_width += *width;
            }

            frozen_cummulative_width = 0.0;
            frozen_cummulative_height = 0.0;
            first_row = true;
            first_column = true;

            for (col, width) in self.widths[0..frozen_columns].iter().enumerate() {
                for (cummulative_height, height, row) in self.heights.iter_range(scrollable_start_y, scrollable_start_y + self.dimension.height - frozen_height) {

                    if row < frozen_rows as u32 {
                        continue;
                    }

                    let cell_borders = self.style.cell_border(row, col as u32);

                    ctx.style(cell_borders.left.style, |ctx| {
                        if first_column {
                            first_column = false;

                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(frozen_cummulative_width + offset_x, cummulative_height),
                                    Position::new(frozen_cummulative_width + offset_x, cummulative_height + height)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.left.width)),
                            );
                        }
                    });

                    ctx.style(cell_borders.right.style, |ctx| {
                        ctx.shape(
                            DrawShape::Line(
                                Position::new(frozen_cummulative_width + offset_x + width, cummulative_height),
                                Position::new(frozen_cummulative_width + offset_x + width, cummulative_height + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.right.width)),
                        );
                    });

                    ctx.style(cell_borders.top.style, |ctx| {
                        if first_row && !has_frozen_rows {
                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(frozen_cummulative_width + offset_x, cummulative_height),
                                    Position::new(frozen_cummulative_width + offset_x + width, cummulative_height)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.top.width)),
                            );
                        }
                    });

                    ctx.style(cell_borders.bottom.style, |ctx| {
                        ctx.shape(
                            DrawShape::Line(
                                Position::new(frozen_cummulative_width + offset_x, cummulative_height + height),
                                Position::new(frozen_cummulative_width + offset_x + width, cummulative_height + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.bottom.width)),
                        );
                    });
                }

                first_column = true;
                first_row = false;
                frozen_cummulative_width += *width;
            }


            // Frozen both
            frozen_cummulative_width = 0.0;
            frozen_cummulative_height = 0.0;

            for (row, height) in self.heights[0..frozen_rows].iter().enumerate() {
                for (col, width) in self.widths[0..frozen_columns].iter().enumerate() {

                    let position = Position::new(frozen_cummulative_width + offset_x, frozen_cummulative_height + offset_y);

                    let cell = self.style.cell(col as u32, row as u32);

                    frozen_cummulative_width += *width;

                    ctx.style(cell.draw_style, |ctx| {
                        ctx.shape(
                            DrawShape::Rectangle(Rect::new(position, Dimension::new(*width, *height))),
                            DrawOptions::Fill(FillOptions::default()),
                        );
                    });

                    if let Some(text) = &cell.text {
                        ctx.text(text, &cell.text_style, Position::new(position.x + 3.0, position.y + 4.0), None);
                    }

                }

                frozen_cummulative_width = 0.0;
                frozen_cummulative_height += *height;
            }

            frozen_cummulative_width = 0.0;
            frozen_cummulative_height = 0.0;
            first_row = true;
            first_column = true;

            for (row, height) in self.heights[0..frozen_rows].iter().enumerate() {
                for (col, width) in self.widths[0..frozen_columns].iter().enumerate() {

                    let cell_borders = self.style.cell_border(row as u32, col as u32);

                    ctx.style(cell_borders.left.style, |ctx| {
                        if first_column {
                            first_column = false;

                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(frozen_cummulative_width + offset_x, frozen_cummulative_height + offset_y),
                                    Position::new(frozen_cummulative_width + offset_x, frozen_cummulative_height + offset_y + height)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.left.width)),
                            );
                        }
                    });

                    ctx.style(cell_borders.right.style, |ctx| {
                        ctx.shape(
                            DrawShape::Line(
                                Position::new(frozen_cummulative_width + offset_x + width, frozen_cummulative_height + offset_y),
                                Position::new(frozen_cummulative_width + offset_x + width, frozen_cummulative_height + offset_y + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.right.width)),
                        );
                    });

                    ctx.style(cell_borders.top.style, |ctx| {
                        if first_row {
                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(frozen_cummulative_width + offset_x, frozen_cummulative_height + offset_y),
                                    Position::new(frozen_cummulative_width + offset_x + width, frozen_cummulative_height + offset_y)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.top.width)),
                            );
                        }
                    });

                    ctx.style(cell_borders.bottom.style, |ctx| {
                        ctx.shape(
                            DrawShape::Line(
                                Position::new(frozen_cummulative_width + offset_x, frozen_cummulative_height + offset_y + height),
                                Position::new(frozen_cummulative_width + offset_x + width, frozen_cummulative_height + offset_y + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.bottom.width)),
                        );
                    });

                    frozen_cummulative_width += *width;
                }

                first_column = true;
                first_row = false;
                frozen_cummulative_width = 0.0;
                frozen_cummulative_height += *height;
            }

        });
    }
}

impl<S: TableStyle + Clone> CommonWidget for Table<S> {
    CommonWidgetImpl!(self, child: (), position: self.position, dimension: self.dimension);
}
