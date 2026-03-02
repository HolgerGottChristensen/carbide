use crate::draw::Rect;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TextDecoration {
    None,
    StrikeThrough,
    Overline,
    Underline,
}