use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::Token;
use crate::carbide_expression::CarbideExpression;
use crate::carbide_item::CarbideItem::{Expression, Struct};
use crate::CarbideStruct;

pub enum CarbideItem {
    Struct(CarbideStruct),
    Expression(CarbideExpression),
}

impl Parse for CarbideItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Token!(struct)) || input.peek2(Token!(struct)) {
            Struct(CarbideStruct::parse(input)?)
        } else {
            Expression(CarbideExpression::parse(input)?)
        })
    }
}

impl ToTokens for CarbideItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Struct(i) => {
                i.to_tokens(tokens)
            }
            Expression(i) => {
                i.to_tokens(tokens)
            }
        }
    }
}