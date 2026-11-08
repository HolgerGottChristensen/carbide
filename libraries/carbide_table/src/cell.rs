use carbide::draw::DrawStyle;
use carbide::text::TextStyle;

pub struct Cell {
    pub draw_style: DrawStyle,
    pub text: Option<String>,
    pub text_style: TextStyle,
    pub resize_handles: ResizeHandles,
}

pub enum ResizeHandles {
    Column,
    Row,
    Both,
    None
}

#[derive(Clone, Debug)]
pub enum CellSelection {
    None,
    Single {
        row: u32,
        column: u32,
    }
}