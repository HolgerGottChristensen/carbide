mod carbide_struct;
mod carbide_expression;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;
use carbide_struct::CarbideStruct;

#[proc_macro]
pub fn CarbideUI(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as CarbideStruct);
    //panic!("\n{:#?}", &input);
    TokenStream::from(input.into_token_stream())
}