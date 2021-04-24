use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::parse::{Parse, ParseBuffer, ParseStream, Result as SynResult};

use crate::widget_node::WidgetNode;
use crate::widget_stmt::WidgetStmt;

#[derive(Debug)]
pub struct WidgetRoot {
    nodes: Vec<WidgetStmt>
}

impl Parse for WidgetRoot {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let mut res = vec![];
        while let Ok(node) = input.parse::<WidgetStmt>() {
            res.push(node);
        }

        Ok(WidgetRoot {
            nodes: res
        })
    }
}

impl ToTokens for WidgetRoot {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(
            quote!(
                pub fn answer() -> u32 {
                    42
                }
            )
        )
    }
}