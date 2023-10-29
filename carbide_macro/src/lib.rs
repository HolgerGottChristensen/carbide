mod carbide_struct;
mod carbide_expression;
mod carbide_gen_optionals;
mod pat_ident_extraction;
mod carbide_item;
mod expr_ident_extraction;
mod dollar_pre_processor;
mod expr;
mod ui;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{FnArg, parse_macro_input};
use carbide_struct::CarbideStruct;
use carbide_syn::Expr;
use crate::carbide_gen_optionals::CarbideGenOptionals;
use crate::carbide_item::CarbideItem;


#[allow(non_snake_case)]
#[proc_macro]
pub fn CarbideUI(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as CarbideItem);

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

    let output = i.sig.output;
    let generics = i.sig.generics;

    let builder = quote!(
        #[automatically_derived]
        pub fn builder #generics (#(#params,)*) #output {
            Self:: #method_name (#(#param_names,)*)
        }

        #[automatically_derived]
        pub fn finish(self) -> Box<Self> {
            Box::new(self)
        }
    );

    //let mut stream = input.into_token_stream();
    //stream.extend(builder);
    //panic!("\n{:#?}", &input.into_token_stream());
    TokenStream::from(builder)
}

#[proc_macro_attribute]
pub fn carbide_default_builder2(_: TokenStream, item: TokenStream) -> TokenStream {
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

    let output = i.sig.output;
    let generics = i.sig.generics;

    let builder = quote!(
        #[automatically_derived]
        pub fn builder #generics (#(#params,)*) #output {
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

#[proc_macro]
pub fn ui(item: TokenStream) -> TokenStream {
    let input = carbide_syn::parse_macro_input!(item as Expr);
    //panic!("\n{:#?}", &input);

    let output = ui::ui(input);

    //panic!("\n{}", &output.into_token_stream());
    TokenStream::from(output.into_token_stream())
}