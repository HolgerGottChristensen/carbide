extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;
extern crate carbide_syn;

use proc_macro::TokenStream;
use quote::ToTokens;
use closure::process_a_expr;

mod derive_type;
mod utils;
mod widget;
mod closure;
mod state_value;
mod identifiable;

#[proc_macro_derive(Widget, attributes(state, id, carbide_derive, carbide_exclude))]
pub fn derive_widget(input: TokenStream) -> TokenStream {
    impl_derive(input, widget::impl_widget)
}


#[proc_macro_derive(StateValue, attributes())]
pub fn derive_state_value(input: TokenStream) -> TokenStream {
    impl_derive(input, state_value::impl_state_value)
}

#[proc_macro_derive(Identifiable, attributes(id))]
pub fn derive_identifiable(input: TokenStream) -> TokenStream {
    impl_derive(input, identifiable::impl_identifiable)
}

// Use the given function to generate a TokenStream for the derive implementation.
fn impl_derive(
    input: TokenStream,
    generate_derive: fn(&syn::DeriveInput) -> proc_macro2::TokenStream,
) -> TokenStream {
    // Parse the input TokenStream representation.
    let ast = syn::parse(input).unwrap();

    // Build the implementation.
    let gen = generate_derive(&ast);
    // Return the generated impl.
    gen.into()
}

#[proc_macro]
pub fn closure(input: TokenStream) -> TokenStream {
    let ast = carbide_syn::parse_macro_input!(input as carbide_syn::Expr);
    let ast_res = process_a_expr(ast);
    //panic!("\n{:#?}", &ast_res);

    TokenStream::from(ast_res.into_token_stream())
}