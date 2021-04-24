use syn::Expr;
use syn::parse::{Parse, ParseBuffer, ParseStream, Result as SynResult};
use syn::token::*;

#[derive(Debug)]
pub struct Value {
    dollar_token: Option<Dollar>,
    expr: Expr,
}

impl Parse for Value {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let dollar = input.parse::<Dollar>().ok();
        let expr = input.parse::<Expr>()?;

        Ok(
            Value {
                dollar_token: dollar,
                expr,
            }
        )
    }
}