use carbide::draw::DrawStyle;
use carbide::text::TextStyle;

pub struct Cell {
    pub draw_style: DrawStyle,
    pub text: Option<String>,
    pub text_style: TextStyle,
}