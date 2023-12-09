use carbide_core::draw::{Position, Scalar};
use carbide_core::text::TextStyle;

pub struct Metadata {
    pub scale_factor: Scalar,
    pub position: Position,
    pub text: String,
    pub style: TextStyle,
}