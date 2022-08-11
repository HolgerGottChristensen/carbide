use std::fmt::{Debug, Formatter};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Error, Ident};
use syn::token::{Brace, Colon, Comma, Dot, In, Let, Paren, Token};
use syn::{braced, Expr, parenthesized, Token, Type};
use syn::__private::{parse_braces, parse_parens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use crate::carbide_expression::CarbideExpression::Instantiate;
use crate::carbide_expression::CarbideInstantiateParam::{Optional, Required};

#[derive(Debug)]
pub enum CarbideExpression {
    Instantiate(CarbideInstantiate),
}

impl ToTokens for CarbideExpression {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Instantiate(i) => {
                tokens.extend(quote!(
                    #i
                ))
            }
        }
    }
}

impl Parse for CarbideExpression {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let instantiate = CarbideInstantiate::parse(input)?;
        Ok(Instantiate(instantiate))
    }
}

pub struct CarbideInstantiate {
    ident: Ident,
    paren: Option<Paren>,
    params: Option<Punctuated<CarbideInstantiateParam, Token![,]>>,
    braces: Option<Brace>,
    iterate: Option<CarbideInstantiateIterate>,
    body: Option<Vec<CarbideExpression>>,
    modifiers: Vec<CarbideInstantiateModifier>,
}

impl ToTokens for CarbideInstantiate {
    fn to_tokens(&self, tokens: &mut TokenStream) {

        let CarbideInstantiate {
            ident,
            body,
            params,
            modifiers,
            ..
        } = self;

        let children = if let Some(body) = body {
            quote!(vec![
                #(#body,)*
            ],)
        } else {
            quote!()
        };

        let required = if let Some(params) = params {

            let mapped = params.iter().filter_map(|param| {
                param.required_init_field()
            });

            quote!(
                #(#mapped,)*
            )
        } else {
            quote!()
        };

        let optional = if let Some(params) = params {

            let mapped = params.iter().filter_map(|param| {
                param.optional_init_function()
            });

            quote!(
                #(#mapped)*
            )
        } else {
            quote!()
        };

        tokens.extend(quote!(
            #ident::new(
                #children
                #required
            )#optional
            #(#modifiers)*
        ))
    }
}

impl Debug for CarbideInstantiate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CarbideInstantiate")
            .field("ident", &self.ident.to_string())
            .field("params", &self.params)
            .field("iterate", &self.iterate)
            .field("body", &self.body)
            .field("modifiers", &self.modifiers)
            .finish()
    }
}

impl Parse for CarbideInstantiate {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = Ident::parse(input)?;


        let paren_content;

        let (paren, params) = if let Ok(paren) = parse_parens(input) {
            paren_content = paren.content;

            let params = {
                use syn::parse_quote::ParseQuote;
                Punctuated::<CarbideInstantiateParam, Token![,]>::parse(&paren_content)?
            };

            (Some(paren.token), Some(params))
        } else {
            (None, None)
        };

        let brace_content;

        let (braces, body, iterate) = if let Ok(brace) = parse_braces(input) {
            brace_content = brace.content;

            let iterate = CarbideInstantiateIterate::parse(&brace_content).ok();

            let mut body = vec![];

            while let Ok(expr) = CarbideExpression::parse(&brace_content) {
                body.push(expr)
            }

            (Some(brace.token), Some(body), iterate)
        } else {
            (None, None, None)
        };

        let mut modifiers = vec![];

        while let Ok(expr) = CarbideInstantiateModifier::parse(input) {
            modifiers.push(expr)
        }

        Ok(CarbideInstantiate {
            ident,
            paren,
            params,
            braces,
            body,
            iterate,
            modifiers
        })
    }
}

pub struct CarbideInstantiateModifier {
    dot_token: Token![.],
    ident: Ident,
    paren_token: Paren,
    exprs: Punctuated<Expr, Comma>,
}

impl ToTokens for CarbideInstantiateModifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {

        let CarbideInstantiateModifier {
            ident,
            exprs,
            ..
        } = self;

        tokens.extend(quote!(
            .#ident(#exprs)
        ))
    }
}

impl Debug for CarbideInstantiateModifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CarbideInstantiateModifier")
            .field("ident", &self.ident.to_string())
            .field("exprs", &self.exprs.to_token_stream().to_string())
            .finish()
    }
}

impl Parse for CarbideInstantiateModifier {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token!(.)) && input.peek2(Ident) && input.peek3(Paren) {
            let dot_token = Dot::parse(input)?;
            let ident = Ident::parse(input)?;

            let paren_content;
            let paren_token = parenthesized!(paren_content in input);

            let exprs = {
                use syn::parse_quote::ParseQuote;
                Punctuated::<Expr, Token![,]>::parse(&paren_content)?
            };

            Ok(CarbideInstantiateModifier {
                dot_token,
                ident,
                paren_token,
                exprs
            })
        } else {
            Err(Error::new(Span::call_site(), "Could not parse carbide modifier"))
        }
    }
}

pub struct CarbideInstantiateIterate {
    vars: Vec<Ident>,
    in_token: Token![in],
}

impl Debug for CarbideInstantiateIterate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CarbideInstantiateIterate")
            .field("ident", &self.vars[0].to_string())
            .finish()
    }
}

impl Parse for CarbideInstantiateIterate {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        let (ident, in_token)= if input.peek(Ident) && input.peek2(Token!(in)) {
            let ident = Ident::parse(input)?;
            let in_token = In::parse(input)?;
            (ident, in_token)
        } else {
            Err(Error::new(Span::call_site(), "Could not parse carbide iterate"))?
        };

        Ok(CarbideInstantiateIterate {
            vars: vec![ident],
            in_token
        })
    }
}

pub enum CarbideInstantiateParam {
    Required {
        expr: Expr,
    },
    Optional {
        ident: Ident,
        colon: Colon,
        expr: Expr,
    }
}

impl CarbideInstantiateParam {
    fn required_init_field(&self) -> Option<TokenStream> {
        match self {
            CarbideInstantiateParam::Required { expr } => {
                Some(quote!({
                    #expr
                }))
            }
            CarbideInstantiateParam::Optional { .. } => None,
        }
    }

    fn optional_init_function(&self) -> Option<TokenStream> {
        match self {
            CarbideInstantiateParam::Optional { ident, expr, .. } => {

                let ident = Ident::new(&format!("with_optional_{}", ident.to_string()), ident.span());

                Some(quote!(
                    .#ident({#expr})
                ))
            }
            CarbideInstantiateParam::Required { .. } => None,
        }
    }
}

impl Debug for CarbideInstantiateParam {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CarbideInstantiateParam::Required { expr } => {
                f.debug_struct("Required")
                    .field("expr", &expr.to_token_stream().to_string())
                    .finish()
            }
            CarbideInstantiateParam::Optional { ident, expr, .. } => {
                f.debug_struct("Optional")
                    .field("ident", &ident.to_string())
                    .field("expr", &expr.to_token_stream().to_string())
                    .finish()
            }
        }
    }
}

impl Parse for CarbideInstantiateParam {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let res = if input.peek(Ident) && input.peek2(Token!(:)) {
            Optional {
                ident: Ident::parse(input)?,
                colon: Colon::parse(input)?,
                expr: Expr::parse(input)?
            }
        } else {
            Required {
                expr: Expr::parse(input)?
            }
        };

        Ok(res)
    }
}