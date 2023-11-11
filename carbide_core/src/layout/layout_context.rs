use crate::draw::InnerImageContext;
use crate::environment::Environment;
use crate::text::InnerTextContext;

pub struct LayoutContext<'a> {
    pub text: &'a mut dyn InnerTextContext,
    pub image: &'a mut dyn InnerImageContext,
    pub env: &'a mut Environment,
}