mod carbide_struct;
mod carbide_expression;
mod carbide_gen_optionals;
mod ident_extraction;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{Expr, FnArg, parse_macro_input, parse_quote};
use carbide_struct::CarbideStruct;
use crate::carbide_gen_optionals::{CarbideGenOptionals};

#[proc_macro]
pub fn CarbideUI(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as CarbideStruct);

    //panic!("\n{:#?}", &input);
    TokenStream::from(input.into_token_stream())
}

#[proc_macro]
pub fn gen_optionals(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as CarbideGenOptionals);
    //panic!("\n{:#?}", &input.into_token_stream());
    TokenStream::from(input.into_token_stream())
}

#[proc_macro_attribute]
pub fn carbide_default_builder(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);

    let i = input.clone();

    let method_name = &i.sig.ident;
    let params = i.sig.inputs.iter();

    let param_names = i.sig.inputs.iter().map(|a| {
        match a {
            FnArg::Receiver(_) => {
                panic!("The default builder must be placed on a method not taking self")
            }
            FnArg::Typed(p) => {
                *p.pat.clone()
            }
        }
    });

    let builder = quote!(
        #[automatically_derived]
        pub fn builder(#(#params,)*) -> Box<Self> {
            Self:: #method_name (#(#param_names,)*)
        }

        #[automatically_derived]
        pub fn finish(self) -> Box<Self> {
            Box::new(self)
        }
    );

    let mut stream = input.into_token_stream();
    stream.extend(builder);
    //panic!("\n{:#?}", &input.into_token_stream());
    TokenStream::from(stream)
}