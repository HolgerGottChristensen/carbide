use proc_macro2::TokenStream;
use std::str::FromStr;
use proc_macro2::{Ident, Span, TokenTree};
use quote::{quote, ToTokens};
use syn::buffer::Cursor;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::parse_quote;
use syn::visit_mut::VisitMut;

pub struct StateAccessReplace;

impl Parse for StateAccessReplace {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        println!("{:?}", input);


        let replaced = input.to_string().replace("$ ", "carbide_state_access__");
        let stream = TokenStream::from_str(&replaced).unwrap();

        println!("{:?}", stream.to_token_stream());

        Ok(StateAccessReplace)
    }
}

#[test]
fn test() {
    let r: StateAccessReplace = parse_quote!(
        +=
    );
}