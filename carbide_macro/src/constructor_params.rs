use syn::parenthesized;
use syn::parse::{Parse, ParseBuffer, ParseStream, Result as SynResult};
use syn::punctuated::Punctuated;
use syn::token::*;

use crate::field::Field;

#[derive(Debug)]
pub struct ConstructorParams {
    paren: Paren,
    fields: Punctuated<Field, Comma>,
}

impl Parse for ConstructorParams {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let content;
        let paren = parenthesized!(content in input);
        let fields = content.parse_terminated(Field::parse)?;

        Ok(
            ConstructorParams {
                paren,
                fields,
            }
        )
    }
}