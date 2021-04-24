use proc_macro2::Ident;
use syn::parse::{Parse, ParseBuffer, ParseStream, Result as SynResult};

use crate::children_params::ChildrenParams;
use crate::constructor_params::ConstructorParams;

#[derive(Debug)]
pub struct WidgetNode {
    ident: Ident,
    constructor_params: Option<ConstructorParams>,
    children_params: Option<ChildrenParams>,
}

impl Parse for WidgetNode {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let ident = input.parse::<Ident>()?;
        let constructor_params = input.parse::<ConstructorParams>().ok();
        //let children_params = input.parse::<ChildrenParams>().ok();

        Ok(
            WidgetNode {
                ident,
                constructor_params,
                children_params: None,
            }
        )
    }
}