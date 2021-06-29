use crate::draw::Rect;
use crate::prelude::Widget;
use crate::widget::GlobalState;

#[derive(Clone, Debug)]
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