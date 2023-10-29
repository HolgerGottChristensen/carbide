use std::fmt::{Debug, Formatter};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Colon, Comma};
use syn::Type;

pub struct CarbideGenOptionals {
    ident: Ident,
    _comma_token: Comma,
    optionals: Punctuated<CarbideGenOptional, Comma>,
}

impl ToTokens for CarbideGenOptionals {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let CarbideGenOptionals {
            ident,
            optionals,
            ..
        } = self;

        let optionals = optionals.iter();

        tokens.extend(quote!(
            /// Generated methods for use with carbide macro optionals
            impl #ident {
                #(#optionals)*
            }
        ))
    }
}

impl Debug for CarbideGenOptionals {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CarbideGenOptionals")
            .field("ident", &self.ident.to_string())
            .field("optionals", &self.optionals)
            .finish()
    }
}

impl Parse for CarbideGenOptionals {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = Ident::parse(input)?;
        let comma_token = Comma::parse(input)?;

        let optionals = {
            use syn::parse_quote::ParseQuote;
            Punctuated::<CarbideGenOptional, Comma>::parse(input)?
        };

        Ok(CarbideGenOptionals {
            ident,
            _comma_token: comma_token,
            optionals
        })
    }
}

pub struct CarbideGenOptional {
    ident: Ident,
    _colon_token: Colon,
    ty: Type
}

impl ToTokens for CarbideGenOptional {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let CarbideGenOptional {
            ident,
            ty,
            ..
        } = self;

        let method_ident = Ident::new(&format!("with_optional_{}", ident.to_string()), ident.span());

        tokens.extend(quote!(
            pub fn #method_ident (mut self, #ident: impl Into<#ty>) -> Box<Self> {
                self. #ident = #ident.into();
                Box::new(self)
            }
        ))
    }
}

impl Debug for CarbideGenOptional {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CarbideGenOptional")
            .field("ident", &self.ident.to_string())
            .field("ty", &self.ty.to_token_stream().to_string())
            .finish()
    }
}

impl Parse for CarbideGenOptional {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = Ident::parse(input)?;
        let colon_token = Colon::parse(input)?;
        let ty = Type::parse(input)?;

        Ok(CarbideGenOptional {
            ident,
            _colon_token: colon_token,
            ty
        })
    }
}