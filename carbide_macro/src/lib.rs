use proc_macro::TokenStream;

use quote::ToTokens;
use syn::parse_macro_input;

use crate::widget_root::WidgetRoot;

mod widget_root;
mod widget_node;
mod constructor_params;
mod children_params;
mod field;
mod value;
mod widget_stmt;
mod widget_expr;
mod widget_expr_if;
mod widget_block;
mod widget_expr_for;
mod widget_binding;

#[proc_macro]
pub fn body(input: TokenStream) -> TokenStream {
    let root = parse_macro_input!(input as WidgetRoot);
    panic!("\n{:#?}", &root);
    TokenStream::from(root.into_token_stream())
}