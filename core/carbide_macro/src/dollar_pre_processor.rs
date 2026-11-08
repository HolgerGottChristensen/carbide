use proc_macro2::TokenStream;
use std::str::FromStr;

use quote::{ToTokens};

use syn::parse::{Parse, ParseStream};



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