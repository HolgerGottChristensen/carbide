use crate::draw::Rect;

#[derive(Clone, Debug, PartialEq)]
pub enum TextDecoration {
    None,
    StrikeThrough(Vec<Rect>),
    Overline(Vec<Rect>),
    Underline(Vec<Rect>),
}

impl TextDecoration {
    pub fn get_rects(&self) -> Vec<Rect> {
        match self {
            TextDecoration::None => vec![],
            TextDecoration::StrikeThrough(r) => r.clone(),
            TextDecoration::Overline(r) => r.clone(),
            TextDecoration::Underline(r) => r.clone(),
        }
    }
}
