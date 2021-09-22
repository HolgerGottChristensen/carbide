use std::fmt::Debug;

use dyn_clone::DynClone;

use crate::prelude::Environment;
use crate::text::text_span::TextSpan;
use crate::text::text_style::TextStyle;

pub trait TextSpanGenerator: DynClone + Debug {
    fn generate(&self, string: &str, style: &TextStyle, env: &mut Environment) -> Vec<TextSpan>;
    fn store_color(&self) -> bool;
}

dyn_clone::clone_trait_object!(TextSpanGenerator);

#[derive(Debug, Clone)]
pub struct NoStyleTextSpanGenerator;

impl TextSpanGenerator for NoStyleTextSpanGenerator {
    fn generate(&self, string: &str, style: &TextStyle, env: &mut Environment) -> Vec<TextSpan> {
        TextSpan::new(string, style, env)
    }

    fn store_color(&self) -> bool {
        false
    }
}
