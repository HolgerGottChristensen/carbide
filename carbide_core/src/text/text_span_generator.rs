use std::fmt::Debug;

use dyn_clone::DynClone;

use crate::prelude::Environment;
use crate::text::text_span::TextSpan;
use crate::text::text_style::TextStyle;
use crate::widget::GlobalStateContract;

pub trait TextSpanGenerator<GS>: DynClone + Debug where GS: GlobalStateContract {
    fn generate(&self, string: &str, style: &TextStyle, env: &mut Environment<GS>) -> Vec<TextSpan<GS>>;
}

dyn_clone::clone_trait_object!(<GS: GlobalStateContract> TextSpanGenerator<GS>);

#[derive(Debug, Clone)]
pub struct NoStyleTextSpanGenerator;

impl<GS: GlobalStateContract> TextSpanGenerator<GS> for NoStyleTextSpanGenerator {
    fn generate(&self, string: &str, style: &TextStyle, env: &mut Environment<GS>) -> Vec<TextSpan<GS>> {
        TextSpan::new(string, style, env)
    }
}
