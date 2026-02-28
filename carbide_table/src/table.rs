use carbide::color::{BLACK, BLUE, GREEN, LIGHT_BLUE, LIGHT_BROWN, LIGHT_GREEN, LIGHT_PURPLE, LIGHT_YELLOW, ORANGE, PURPLE, RED, WHITE, YELLOW};
use carbide::draw::fill::FillOptions;
use carbide::draw::{DrawOptions, DrawShape, DrawStyle, Rect, Scalar};
use carbide_core::draw::{Dimension, Position};
use carbide_core::event::{MouseEvent, MouseEventContext, MouseEventHandler};
use carbide_core::layout::{Layout, LayoutContext};
use carbide_core::render::{Render, RenderContext};
use carbide_core::widget::{CommonWidget, Widget, WidgetExt, WidgetId};
use carbide_core::CommonWidgetImpl;
use std::fmt::Debug;
use carbide::draw::stroke::StrokeOptions;
use carbide::math::{Matrix4, Vector3};
use carbide::state::{LocalState, ReadState, State};
use crate::size_collection::SizeCollection;

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(MouseEvent, Render, Layout)]
pub struct Table {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,
    offset: LocalState<Position>,
    widths: Vec<f64>,
    heights: Vec<f64>,
    frozen_columns: LocalState<usize>,
    frozen_rows: LocalState<usize>,
}

impl Table {
    pub fn new() -> Table {
        Table {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            offset: LocalState::new(Position::origin()),
            widths: vec![80.0; 40],
            heights: vec![25.0; 40],
            frozen_columns: LocalState::new(1),
            frozen_rows: LocalState::new(1),
        }
    }
}

impl MouseEventHandler for Table {
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

impl Layout for Table {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.set_dimension(requested_size);
        requested_size
    }
}

impl Render for Table {
    fn render(&mut self, ctx: &mut RenderContext) {
        let offset_x = self.offset.value().x;
        let offset_y = self.offset.value().y;

        let scroll_x = self.position.x - offset_x;
        let scroll_y = self.position.y - offset_y;

        let frozen_columns = *self.frozen_columns.value();
        let frozen_rows = *self.frozen_rows.value();

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

                    let color = if (row + col) % 2 == 0 { RED } else { BLUE };

                    ctx.style(DrawStyle::Color(color), |ctx| {
                        ctx.shape(
                            DrawShape::Rectangle(Rect::new(
                                Position::new(cumulative_width, cumulative_height),
                                Dimension::new(width, height)
                            )),
                            DrawOptions::Fill(FillOptions::default()),
                        );
                    })
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

                    ctx.style(DrawStyle::Color(WHITE), |ctx| {
                        if first_column && !has_frozen_columns {
                            first_column = false;

                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(cummulative_width, cummulative_height),
                                    Position::new(cummulative_width, cummulative_height + height)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(1.0)),
                            );
                        }

                        ctx.shape(
                            DrawShape::Line(
                                Position::new(cummulative_width + width, cummulative_height),
                                Position::new(cummulative_width + width, cummulative_height + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(1.0)),
                        );
                    });

                    ctx.style(DrawStyle::Color(WHITE), |ctx| {
                        if first_row && !has_frozen_rows {
                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(cummulative_width, cummulative_height),
                                    Position::new(cummulative_width + width, cummulative_height)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(1.0)),
                            );
                        }

                        ctx.shape(
                            DrawShape::Line(
                                Position::new(cummulative_width, cummulative_height + height),
                                Position::new(cummulative_width + width, cummulative_height + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(1.0)),
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

                    let color = if (row as u32 + col) % 2 == 0 { YELLOW } else { GREEN };

                    ctx.style(DrawStyle::Color(color), |ctx| {
                        ctx.shape(
                            DrawShape::Rectangle(Rect::new(position, Dimension::new(width, *height))),
                            DrawOptions::Fill(FillOptions::default()),
                        );
                    })
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

                    ctx.style(DrawStyle::Color(WHITE), |ctx| {
                        if first_column && !has_frozen_columns {
                            first_column = false;

                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(cummulative_width, frozen_cummulative_height + offset_y),
                                    Position::new(cummulative_width, frozen_cummulative_height + offset_y + height)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(1.0)),
                            );
                        }

                        ctx.shape(
                            DrawShape::Line(
                                Position::new(cummulative_width + width, frozen_cummulative_height + offset_y),
                                Position::new(cummulative_width + width, frozen_cummulative_height + offset_y + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(1.0)),
                        );
                    });

                    ctx.style(DrawStyle::Color(WHITE), |ctx| {
                        if first_row {
                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(cummulative_width, frozen_cummulative_height + offset_y),
                                    Position::new(cummulative_width + width, frozen_cummulative_height + offset_y)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(1.0)),
                            );
                        }

                        ctx.shape(
                            DrawShape::Line(
                                Position::new(cummulative_width, frozen_cummulative_height + offset_y + height),
                                Position::new(cummulative_width + width, frozen_cummulative_height + offset_y + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(1.0)),
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

                    let color = if (row + col as u32) % 2 == 0 { YELLOW } else { GREEN };

                    ctx.style(DrawStyle::Color(color), |ctx| {
                        ctx.shape(
                            DrawShape::Rectangle(Rect::new(position, Dimension::new(*width, height))),
                            DrawOptions::Fill(FillOptions::default()),
                        );
                    })
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

                    ctx.style(DrawStyle::Color(WHITE), |ctx| {
                        if first_column {
                            first_column = false;

                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(frozen_cummulative_width + offset_x, cummulative_height),
                                    Position::new(frozen_cummulative_width + offset_x, cummulative_height + height)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(1.0)),
                            );
                        }

                        ctx.shape(
                            DrawShape::Line(
                                Position::new(frozen_cummulative_width + offset_x + width, cummulative_height),
                                Position::new(frozen_cummulative_width + offset_x + width, cummulative_height + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(1.0)),
                        );
                    });

                    ctx.style(DrawStyle::Color(WHITE), |ctx| {
                        if first_row && !has_frozen_rows {
                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(frozen_cummulative_width + offset_x, cummulative_height),
                                    Position::new(frozen_cummulative_width + offset_x + width, cummulative_height)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(1.0)),
                            );
                        }

                        ctx.shape(
                            DrawShape::Line(
                                Position::new(frozen_cummulative_width + offset_x, cummulative_height + height),
                                Position::new(frozen_cummulative_width + offset_x + width, cummulative_height + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(1.0)),
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

                    let color = if (row + col) % 2 == 0 { PURPLE } else { ORANGE };;

                    frozen_cummulative_width += *width;

                    ctx.style(DrawStyle::Color(color), |ctx| {
                        ctx.shape(
                            DrawShape::Rectangle(Rect::new(position, Dimension::new(*width, *height))),
                            DrawOptions::Fill(FillOptions::default()),
                        );
                    })
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

                    ctx.style(DrawStyle::Color(WHITE), |ctx| {
                        if first_column {
                            first_column = false;

                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(frozen_cummulative_width + offset_x, frozen_cummulative_height + offset_y),
                                    Position::new(frozen_cummulative_width + offset_x, frozen_cummulative_height + offset_y + height)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(1.0)),
                            );
                        }

                        ctx.shape(
                            DrawShape::Line(
                                Position::new(frozen_cummulative_width + offset_x + width, frozen_cummulative_height + offset_y),
                                Position::new(frozen_cummulative_width + offset_x + width, frozen_cummulative_height + offset_y + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(1.0)),
                        );
                    });

                    ctx.style(DrawStyle::Color(WHITE), |ctx| {
                        if first_row {
                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(frozen_cummulative_width + offset_x, frozen_cummulative_height + offset_y),
                                    Position::new(frozen_cummulative_width + offset_x + width, frozen_cummulative_height + offset_y)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(1.0)),
                            );
                        }

                        ctx.shape(
                            DrawShape::Line(
                                Position::new(frozen_cummulative_width + offset_x, frozen_cummulative_height + offset_y + height),
                                Position::new(frozen_cummulative_width + offset_x + width, frozen_cummulative_height + offset_y + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(1.0)),
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

impl CommonWidget for Table {
    CommonWidgetImpl!(self, child: (), position: self.position, dimension: self.dimension);
}
