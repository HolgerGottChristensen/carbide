use proc_macro2::Ident;
use syn::{Error as SynError, Expr};
use syn::parse::{Parse, ParseBuffer, ParseStream, Result as SynResult};
use syn::token::*;

use crate::value::Value;

#[derive(Debug)]
pub enum Field {
    Named(NamedField),
    UnNamed(UnNamedField),
}

impl Parse for Field {
    fn parse(input: ParseStream) -> SynResult<Self> {
        if let Ok(res) = input.parse::<NamedField>() {
            Ok(Field::Named(res))
        } else if let Ok(res) = input.parse::<UnNamedField>() {
            Ok(Field::UnNamed(res))
        } else {
            Err(SynError::new(input.span(), "could not parse field"))
        }
    }
}

#[derive(Debug)]
pub struct NamedField {
    ident: Ident,
    colon_token: Colon,
    value: Value,
}

impl Parse for NamedField {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let ident = input.parse::<Ident>()?;
        let colon = input.parse::<Colon>()?;
        let value = input.parse::<Value>()?;

        Ok(
            NamedField {
                ident,
                colon_token: colon,
                value,
            }
        )
    }
}

#[derive(Debug)]
pub struct UnNamedField {
    value: Value
}

impl Parse for UnNamedField {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let value = input.parse::<Value>()?;

        Ok(
            UnNamedField {
                value
            }
        )
    }
}