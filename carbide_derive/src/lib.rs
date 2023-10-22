extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;
extern crate carbide_syn;

use proc_macro::TokenStream;
use quote::ToTokens;
use a::process_a_expr;
use carbide_syn::fold::Fold;

mod common;
mod derive_type;
mod style;
mod utils;
mod widget;
mod state;
mod a;

// The implementation for the `WidgetCommon` trait derivation (aka `carbide_core::widget::Common`).
#[proc_macro_derive(WidgetCommon, attributes(carbide, common_builder))]
pub fn widget_common(input: TokenStream) -> TokenStream {
    impl_derive(input, common::impl_widget_common)
}

// The implementation for the `WidgetCommon_` trait derivation (aka `carbide_core::widget::Common`).
//
// Note that this is identical to the `WidgetCommon` trait, but only for use within the carbide
// crate itself.
#[proc_macro_derive(WidgetCommon_, attributes(carbide, common_builder))]
pub fn widget_common_(input: TokenStream) -> TokenStream {
    impl_derive(input, common::impl_widget_common_)
}

// The implementation for the `WidgetStyle` trait derivation (aka `carbide_core::widget::Style`).
#[proc_macro_derive(WidgetStyle, attributes(carbide, default))]
pub fn widget_style(input: TokenStream) -> TokenStream {
    impl_derive(input, style::impl_widget_style)
}

// The implementation for the `WidgetStyle_` trait derivation (aka `carbide_core::widget::Style`).
//
// Note that this is identical to the `WidgetStyle_` trait, but only for use within the carbide
// crate itself.
#[proc_macro_derive(WidgetStyle_, attributes(carbide, default))]
pub fn widget_style_(input: TokenStream) -> TokenStream {
    impl_derive(input, style::impl_widget_style_)
}

#[proc_macro_derive(Widget, attributes(state, carbide_derive, carbide_exclude))]
pub fn derive_widget(input: TokenStream) -> TokenStream {
    impl_derive(input, widget::impl_widget)
}

#[proc_macro_derive(State, attributes())]
pub fn derive_state(input: TokenStream) -> TokenStream {
    impl_derive(input, state::impl_state)
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
pub fn a(input: TokenStream) -> TokenStream {
    let ast = carbide_syn::parse_macro_input!(input as carbide_syn::Expr);
    let ast_res = process_a_expr(ast);
    //panic!("\n{:#?}", &ast_res);

    TokenStream::from(ast_res.into_token_stream())
}