use crate::cell::{CellSelection, ResizeHandles};
use crate::size_collection::SizeCollection;
use crate::style::{SpreadsheetStyle, TableStyle};
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
use carbide::cursor::MouseCursor;

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(MouseEvent, Render, Layout)]
pub struct Table<S> where S: TableStyle + Clone {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,
    offset: LocalState<Position>,

    style: S,

    resizing: CellResizing
}

impl Table<SpreadsheetStyle> {
    pub fn new() -> Table<SpreadsheetStyle> {
        Table {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            offset: LocalState::new(Position::origin()),

            style: SpreadsheetStyle {
                frozen_columns: LocalState::new(1),
                frozen_rows: LocalState::new(1),
                hovered_cell: LocalState::new(CellSelection::Single { row: 5, column: 5 }),
                widths: vec![100.0; 40],
                heights: vec![21.0; 100],
            },
            resizing: CellResizing::None,
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

                new_x = new_x.clamp(0.0, (self.style.widths().iter().sum::<f64>() - self.dimension.width).max(0.0));
                new_y = new_y.clamp(0.0, (self.style.heights().iter().sum::<f64>() - self.dimension.height).max(0.0));

                self.offset.value_mut().y = new_y;
                self.offset.value_mut().x = new_x;

                self.handle_mouse_position_inside_table_change(mouse_position);
            }
            MouseEvent::Move { to, .. } => {
                if !self.is_inside(*to) {
                    self.style.set_hovered(CellSelection::None);
                    return;
                }

                self.handle_mouse_position_inside_table_change(to);
            }
            MouseEvent::Drag { button, origin, from, to, delta_xy, total_delta_xy, modifiers } => {
                match self.resizing {
                    CellResizing::None | CellResizing::BothHovered |CellResizing::RowHovered | CellResizing::ColHovered => {
                        let (resize_col, resize_row) = self.resize_handles_from_position(origin);

                        if let Some(r) = resize_row {
                            self.resizing = CellResizing::Row(r, self.style.heights()[r as usize]);
                        }

                        if let Some(c) = resize_col {
                            self.resizing = CellResizing::Col(c, self.style.widths()[c as usize]);
                        }

                        if let Some(r) = resize_row && let Some(c) = resize_col {
                            self.resizing = CellResizing::Both {
                                col: c,
                                row: r,
                                cell_dimension: Dimension::new(
                                    self.style.widths()[c as usize],
                                    self.style.heights()[r as usize]
                                )
                            }
                        }

                        if resize_col.is_none() && resize_row.is_none() {
                            self.resizing = CellResizing::NoneWhileDragging;
                        }

                    }
                    CellResizing::Both { col, row, cell_dimension } => {
                        let new_width = (cell_dimension.width + total_delta_xy.x).max(15.0);
                        let new_height = (cell_dimension.height + total_delta_xy.y).max(15.0);

                        self.style.resize_cell(col, row, Dimension::new(new_width, new_height));
                    }
                    CellResizing::Col(col, cell_width) => {
                        let new_width = (cell_width + total_delta_xy.x).max(15.0);
                        self.style.resize_width(col, new_width);
                    }
                    CellResizing::Row(row, cell_height) => {
                        let new_height = (cell_height + total_delta_xy.y).max(15.0);
                        self.style.resize_height(row, new_height);
                    }
                    _ => {}
                }
            }
            MouseEvent::Release { .. } => {
                self.resizing = CellResizing::None;
            }
            _ => ()
        }
    }
}

impl<S: TableStyle + Clone> Table<S> {
    fn handle_mouse_position_inside_table_change(&mut self, to: &Position) {
        let TableCellInfo { y_in_cell, height, row, x_in_cell, width, col } = self.cell_from_position(to);

        self.style.set_hovered(CellSelection::Single { row, column: col });

        match self.resizing {
            CellResizing::NoneWhileDragging |
            CellResizing::Both { .. } |
            CellResizing::Col(_, _) |
            CellResizing::Row(_, _) => {}
            _ => {
                let (resize_col, resize_row) = self.resize_handles_from_position(to);

                if let Some(r) = resize_row {
                    self.resizing = CellResizing::RowHovered;
                }

                if let Some(c) = resize_col {
                    self.resizing = CellResizing::ColHovered;
                }

                if let Some(r) = resize_row && let Some(c) = resize_col {
                    self.resizing = CellResizing::BothHovered;
                }

                if resize_row.is_none() && resize_col.is_none() {
                    self.resizing = CellResizing::None;
                }
            }
        }
    }
}

impl<S: TableStyle + Clone> Table<S> {
    fn resize_handles_from_position(&mut self, origin: &Position) -> (Option<u32>, Option<u32>) {
        let TableCellInfo { y_in_cell, height, row, x_in_cell, width, col } = self.cell_from_position(origin);

        let resize_padding = 4.0;

        let mut resize_col = None;
        let mut resize_row = None;

        if width - x_in_cell < resize_padding {
            let cell = self.style.cell(col, row);

            if matches!(cell.resize_handles, ResizeHandles::Column | ResizeHandles::Both) {
                resize_col = Some(col);
            }
        }

        if x_in_cell < resize_padding && col >= 1 {
            let cell = self.style.cell(col - 1, row);

            if matches!(cell.resize_handles, ResizeHandles::Column | ResizeHandles::Both) {
                resize_col = Some(col - 1);
            }
        }

        if height - y_in_cell < resize_padding {
            let cell = self.style.cell(col, row);

            if matches!(cell.resize_handles, ResizeHandles::Row | ResizeHandles::Both) {
                resize_row = Some(row);
            }
        }

        if y_in_cell < resize_padding && row >= 1 {
            let cell = self.style.cell(col, row - 1);

            if matches!(cell.resize_handles, ResizeHandles::Row | ResizeHandles::Both) {
                resize_row = Some(row - 1);
            }
        }

        if y_in_cell < resize_padding && row >= 1 && x_in_cell < resize_padding && col >= 1 {
            let cell = self.style.cell(col - 1, row - 1);

            if matches!(cell.resize_handles, ResizeHandles::Both) {
                resize_row = Some(row - 1);
            }

            if matches!(cell.resize_handles, ResizeHandles::Both) {
                resize_col = Some(col - 1);
            }
        }

        if height - y_in_cell < resize_padding && x_in_cell < resize_padding && col >= 1 {
            let cell = self.style.cell(col - 1, row);

            if matches!(cell.resize_handles, ResizeHandles::Both) {
                resize_row = Some(row);
            }

            if matches!(cell.resize_handles, ResizeHandles::Both) {
                resize_col = Some(col - 1);
            }
        }

        if y_in_cell < resize_padding && row >= 1 && width - x_in_cell < resize_padding {
            let cell = self.style.cell(col, row - 1);

            if matches!(cell.resize_handles, ResizeHandles::Both) {
                resize_row = Some(row - 1);
            }

            if matches!(cell.resize_handles, ResizeHandles::Both) {
                resize_col = Some(col);
            }
        }

        (resize_col, resize_row)
    }
}

impl<S: TableStyle + Clone> Table<S> {
    fn cell_from_position(&mut self, position: &Position) -> TableCellInfo {
        let offset_x = self.offset.value().x;
        let offset_y = self.offset.value().y;

        let pos_in_table_x = position.x - self.position.x;
        let pos_in_table_y = position.y - self.position.y;

        let scroll_offset_x = pos_in_table_x + offset_x;
        let scroll_offset_y = pos_in_table_y + offset_y;

        let frozen_columns = self.style.frozen_columns();
        let frozen_rows = self.style.frozen_rows();

        let frozen_width = self.style.widths()[0..frozen_columns].iter().sum::<Scalar>();
        let frozen_height = self.style.heights()[0..frozen_rows].iter().sum::<Scalar>();

        let mut y_in_cell = 0.0;
        let mut height = 0.0;
        let mut row = 0;

        let mut x_in_cell = 0.0;
        let mut width = 0.0;
        let mut col = 0;

        if pos_in_table_y < frozen_height {
            self.style.heights().iter_range(pos_in_table_y, pos_in_table_y).next().map(|(inner_y, inner_height, inner_row)| {
                y_in_cell = pos_in_table_y - inner_y;
                height = inner_height;
                row = inner_row;
            });
        } else {
            self.style.heights().iter_range(scroll_offset_y, scroll_offset_y).next().map(|(inner_y, inner_height, inner_row)| {
                y_in_cell = scroll_offset_y - inner_y;
                height = inner_height;
                row = inner_row;
            });
        }

        if pos_in_table_x < frozen_width {
            self.style.widths().iter_range(pos_in_table_x, pos_in_table_x).next().map(|(inner_x, inner_width, inner_col)| {
                x_in_cell = pos_in_table_x - inner_x;
                width = inner_width;
                col = inner_col;
            });
        } else {
            self.style.widths().iter_range(scroll_offset_x, scroll_offset_x).next().map(|(inner_x, inner_width, inner_col)| {
                x_in_cell = scroll_offset_x - inner_x;
                width = inner_width;
                col = inner_col;
            });
        }

        TableCellInfo {
            y_in_cell,
            height,
            row,
            x_in_cell,
            width,
            col
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

        let frozen_width = self.style.widths()[0..frozen_columns].iter().sum::<Scalar>();
        let frozen_height = self.style.heights()[0..frozen_rows].iter().sum::<Scalar>();

        let scrollable_start_x = offset_x + frozen_width;
        let scrollable_start_y = offset_y + frozen_height;

        let scrollable_end_y = scrollable_start_y + self.dimension.height - frozen_height;
        let scrollable_end_x = scrollable_start_x + self.dimension.width - frozen_width;

        // The transform handles the scrolling of the grid
        ctx.transform(Matrix4::from_translation(Vector3::new(scroll_x as f32, scroll_y as f32, 0.0)), |ctx| {
            // Draw the backgrounds for each visible moving cell
            for (cumulative_height, height, row) in self.style.heights().iter_range(scrollable_start_y, scrollable_end_y) {
                for (cumulative_width, width, col) in self.style.widths().iter_range(scrollable_start_x, scrollable_end_x) {

                    // We only draw moving cells, so we skip the frozen rows and columns
                    if row < frozen_rows as u32 || col < frozen_columns as u32 {
                        continue;
                    }

                    self.style.draw_cell(col, row, Rect::new(
                        Position::new(cumulative_width, cumulative_height),
                        Dimension::new(width, height)
                    ), ctx);
                }
            }

            let mut first_column = true;
            let mut first_row = true;

            // Draw vertical and horizontal lines for moving visible cells
            for (cummulative_height, height, row) in self.style.heights().iter_range(scrollable_start_y, scrollable_end_y) {
                for (cummulative_width, width, col) in self.style.widths().iter_range(scrollable_start_x, scrollable_end_x) {

                    if row < frozen_rows as u32 || col < frozen_columns as u32 {
                        continue;
                    }

                    let cell_borders = self.style.cell_border(row, col);

                    if first_column && !has_frozen_columns {
                        first_column = false;
                        ctx.style(cell_borders.left.style, |ctx| {
                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(cummulative_width, cummulative_height),
                                    Position::new(cummulative_width, cummulative_height + height)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.left.width)),
                            );
                        });
                    }


                    ctx.style(cell_borders.right.style, |ctx| {
                        ctx.shape(
                            DrawShape::Line(
                                Position::new(cummulative_width + width, cummulative_height),
                                Position::new(cummulative_width + width, cummulative_height + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.right.width)),
                        );
                    });


                    if first_row && !has_frozen_rows {
                        ctx.style(cell_borders.top.style, |ctx| {
                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(cummulative_width, cummulative_height),
                                    Position::new(cummulative_width + width, cummulative_height)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.top.width)),
                            );
                        });
                    }


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
            for (row, height) in self.style.heights()[0..frozen_rows].iter().enumerate() {
                for (cummulative_width, width, col) in self.style.widths().iter_range(scrollable_start_x, scrollable_start_x + self.dimension.width - frozen_width) {

                    if col < frozen_columns as u32 {
                        continue;
                    }

                    self.style.draw_cell(col, row as u32, Rect::new(
                        Position::new(cummulative_width, frozen_cummulative_height + offset_y),
                        Dimension::new(width, *height)
                    ), ctx);
                }

                frozen_cummulative_height += *height;
            }

            frozen_cummulative_width = 0.0;
            frozen_cummulative_height = 0.0;
            first_row = true;
            first_column = true;

            for (row, height) in self.style.heights()[0..frozen_rows].iter().enumerate() {
                for (cummulative_width, width, col) in self.style.widths().iter_range(scrollable_start_x, scrollable_start_x + self.dimension.width - frozen_width) {

                    if col < frozen_columns as u32 {
                        continue;
                    }

                    let cell_borders = self.style.cell_border(row as u32, col);


                    if first_column && !has_frozen_columns {
                        first_column = false;
                        ctx.style(cell_borders.left.style, |ctx| {
                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(cummulative_width, frozen_cummulative_height + offset_y),
                                    Position::new(cummulative_width, frozen_cummulative_height + offset_y + height)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.left.width)),
                            );
                        });
                    }


                    ctx.style(cell_borders.right.style, |ctx| {
                        ctx.shape(
                            DrawShape::Line(
                                Position::new(cummulative_width + width, frozen_cummulative_height + offset_y),
                                Position::new(cummulative_width + width, frozen_cummulative_height + offset_y + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.right.width)),
                        );
                    });


                    if first_row {
                        ctx.style(cell_borders.top.style, |ctx| {
                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(cummulative_width, frozen_cummulative_height + offset_y),
                                    Position::new(cummulative_width + width, frozen_cummulative_height + offset_y)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.top.width)),
                            );
                        });
                    }


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

            for (col, width) in self.style.widths()[0..frozen_columns].iter().enumerate() {
                for (cummulative_height, height, row) in self.style.heights().iter_range(scrollable_start_y, scrollable_start_y + self.dimension.height - frozen_height) {

                    if row < frozen_rows as u32 {
                        continue;
                    }

                    self.style.draw_cell(col as u32, row, Rect::new(
                        Position::new(frozen_cummulative_width + offset_x, cummulative_height),
                        Dimension::new(*width, height)
                    ), ctx);
                }

                frozen_cummulative_width += *width;
            }

            frozen_cummulative_width = 0.0;
            frozen_cummulative_height = 0.0;
            first_row = true;
            first_column = true;

            for (col, width) in self.style.widths()[0..frozen_columns].iter().enumerate() {
                for (cummulative_height, height, row) in self.style.heights().iter_range(scrollable_start_y, scrollable_start_y + self.dimension.height - frozen_height) {

                    if row < frozen_rows as u32 {
                        continue;
                    }

                    let cell_borders = self.style.cell_border(row, col as u32);


                    if first_column {
                        first_column = false;

                        ctx.style(cell_borders.left.style, |ctx| {
                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(frozen_cummulative_width + offset_x, cummulative_height),
                                    Position::new(frozen_cummulative_width + offset_x, cummulative_height + height)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.left.width)),
                            );
                        });
                    }


                    ctx.style(cell_borders.right.style, |ctx| {
                        ctx.shape(
                            DrawShape::Line(
                                Position::new(frozen_cummulative_width + offset_x + width, cummulative_height),
                                Position::new(frozen_cummulative_width + offset_x + width, cummulative_height + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.right.width)),
                        );
                    });


                    if first_row && !has_frozen_rows {
                        ctx.style(cell_borders.top.style, |ctx| {
                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(frozen_cummulative_width + offset_x, cummulative_height),
                                    Position::new(frozen_cummulative_width + offset_x + width, cummulative_height)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.top.width)),
                            );
                        });
                    }


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

            for (row, height) in self.style.heights()[0..frozen_rows].iter().enumerate() {
                for (col, width) in self.style.widths()[0..frozen_columns].iter().enumerate() {

                    self.style.draw_cell(col as u32, row as u32, Rect::new(
                        Position::new(frozen_cummulative_width + offset_x, frozen_cummulative_height + offset_y),
                        Dimension::new(*width, *height)
                    ), ctx);

                    frozen_cummulative_width += *width;
                }

                frozen_cummulative_width = 0.0;
                frozen_cummulative_height += *height;
            }

            frozen_cummulative_width = 0.0;
            frozen_cummulative_height = 0.0;
            first_row = true;
            first_column = true;

            for (row, height) in self.style.heights()[0..frozen_rows].iter().enumerate() {
                for (col, width) in self.style.widths()[0..frozen_columns].iter().enumerate() {

                    let cell_borders = self.style.cell_border(row as u32, col as u32);

                    if first_column {
                        first_column = false;

                        ctx.style(cell_borders.left.style, |ctx| {
                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(frozen_cummulative_width + offset_x, frozen_cummulative_height + offset_y),
                                    Position::new(frozen_cummulative_width + offset_x, frozen_cummulative_height + offset_y + height)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.left.width)),
                            );
                        });
                    }


                    ctx.style(cell_borders.right.style, |ctx| {
                        ctx.shape(
                            DrawShape::Line(
                                Position::new(frozen_cummulative_width + offset_x + width, frozen_cummulative_height + offset_y),
                                Position::new(frozen_cummulative_width + offset_x + width, frozen_cummulative_height + offset_y + height)
                            ),
                            DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.right.width)),
                        );
                    });


                    if first_row {
                        ctx.style(cell_borders.top.style, |ctx| {
                            ctx.shape(
                                DrawShape::Line(
                                    Position::new(frozen_cummulative_width + offset_x, frozen_cummulative_height + offset_y),
                                    Position::new(frozen_cummulative_width + offset_x + width, frozen_cummulative_height + offset_y)
                                ),
                                DrawOptions::Stroke(StrokeOptions::default().with_stroke_width(cell_borders.top.width)),
                            );
                        });
                    }


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

        match self.resizing {
            CellResizing::Both { .. } | CellResizing::BothHovered => {
                if let Some(env_cursor) = ctx.env.get_mut::<MouseCursor>() {
                    *env_cursor = MouseCursor::NwseResize;
                }
            }
            CellResizing::Col(_, _) | CellResizing::ColHovered => {
                if let Some(env_cursor) = ctx.env.get_mut::<MouseCursor>() {
                    *env_cursor = MouseCursor::ColResize;
                }
            }
            CellResizing::Row(_, _) | CellResizing::RowHovered => {
                if let Some(env_cursor) = ctx.env.get_mut::<MouseCursor>() {
                    *env_cursor = MouseCursor::RowResize;
                }
            }
            _ => {}
        }
    }
}

impl<S: TableStyle + Clone> CommonWidget for Table<S> {
    CommonWidgetImpl!(self, child: (), position: self.position, dimension: self.dimension);
}

struct TableCellInfo {
    y_in_cell: f64,
    height: f64,
    row: u32,
    x_in_cell: f64,
    width: f64,
    col: u32
}

#[derive(Clone, Debug)]
enum CellResizing {
    None,
    NoneWhileDragging,
    Both { col: u32, row: u32, cell_dimension: Dimension },
    BothHovered,
    Col(u32, Scalar),
    ColHovered,
    Row(u32, Scalar),
    RowHovered,
}