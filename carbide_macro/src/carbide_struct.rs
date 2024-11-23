use std::fmt::{Debug, Formatter};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::token::{Brace, Colon, Let, Paren, Struct};
use syn::{Attribute, braced, Expr, ItemFn, parse_quote, Token, Type};
use syn::__private::parse_parens;
use syn::parse::{Parse, ParseStream};

use syn::token::parsing::keyword;
use crate::carbide_expression::{CarbideBlock};
use crate::carbide_struct::CarbideStructField::{Required, Optional};


pub struct CarbideStruct {
    _struct_token: Token![struct],
    ident: Ident,
    _brace: Brace,
    fields: Vec<CarbideStructField>,
    body: CarbideBodyFunction,
}

impl Debug for CarbideStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CarbideStruct")
            .field("ident", &self.ident.to_string())
            .field("fields", &self.fields)
            .field("body", &self.body)
            .finish()
    }
}

impl Parse for CarbideStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let struct_token = Struct::parse(input)?;
        let ident = Ident::parse(input)?;
        let content;
        let brace = braced!(content in input);

        let mut fields = vec![];
        while let Ok(field) = CarbideStructField::parse(&content) {
            fields.push(field);
        }

        let body = CarbideBodyFunction::parse(&content)?;

        Ok(CarbideStruct {
            _struct_token: struct_token,
            ident,
            _brace: brace,
            fields,
            body
        })
    }
}

impl ToTokens for CarbideStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {

        let CarbideStruct {
            ident,
            fields,
            body,
            ..
        } = self;

        let struct_fields = fields.iter().map(|field| {
            field.to_struct_field()
        });

        let struct_builder_fields = fields.iter().map(|field| {
            field.to_struct_builder_field()
        });

        let optional_field_methods = fields.iter().filter_map(|field| {
            field.to_optional_function()
        });

        let required_fields_args = fields.iter().filter_map(|field| {
            field.to_required_arg()
        });

        let required_fields_arg_names = fields.iter().filter_map(|field| {
            field.to_required_arg_name()
        });

        let required_fields_args2 = required_fields_args.clone();

        let struct_init_fields = fields.iter().map(|field| {
            field.to_struct_init_field()
        });

        let struct_fields_use = fields.iter().map(|field| {
            field.to_struct_field_names()
        });

        let struct_fields_use2 = struct_fields_use.clone();

        // If we have a body with multiple widget returns we store them in a field in the struct
        let child_field = quote!(child: Box<dyn AnyWidget>,);

        let child_init_field = quote!(child: child,);

        let children_let = quote!(
                let child = #body;
            );

        let children_common = quote!(
                child: self.child,
            );

        let builder_ident = Ident::new(&format!("{}Builder", &ident.to_string()), ident.span());

        tokens.extend(quote!(
            /// A widget generated by the CarbideUI macro
            #[derive(Clone, Debug, Widget)]
            pub struct #ident {
                id: WidgetId,
                position: Position,
                dimension: Dimension,
                #(#struct_fields, )*
                #child_field
            }

            /// Body new function
            impl #ident {
                pub fn new(#(#required_fields_args, )*) -> Box<Self> {
                    #ident::builder(#(#required_fields_arg_names, )*).finish()
                }
            }

            /// The builder struct
            pub struct #builder_ident {
                #(#struct_builder_fields, )*
            }

            /// Create a builder
            impl #ident {
                pub fn builder(#(#required_fields_args2, )*) -> #builder_ident {
                    #builder_ident {
                        #(#struct_init_fields, )*
                    }
                }
            }

            /// Optional fields impl
            impl #builder_ident {
                #(#optional_field_methods)*
            }

            /// Builder finish, construct the actual widget
            impl #builder_ident {
                pub fn finish(self) -> Box<#ident> {
                    let #builder_ident {
                        #(#struct_fields_use,)*
                    } = self;

                    #children_let

                    Box::new(#ident {
                        id: WidgetId::new(),
                        position: Position::new(0.0, 0.0),
                        dimension: Dimension::new(100.0, 100.0),
                        #(#struct_fields_use2,)*
                        #child_init_field
                    })
                }
            }


            impl carbide_core::widget::CommonWidget for #ident {
                carbide_core::CommonWidgetImpl!(self, #children_common position: self.position, dimension: self.dimension);
            }
        ))
    }
}

pub struct CarbideBodyFunction {
    _fn_token: Token![fn],
    _ident: Ident,
    _parenthesis: Paren,
    _arrow: Token![->],
    return_type: Type,
    body: CarbideBlock,
}

impl ToTokens for CarbideBodyFunction {
    fn to_tokens(&self, tokens: &mut TokenStream) {

        let CarbideBodyFunction {
            body,
            ..
        } = self;

        let body = if body.produces_vec() {
            quote!(
                ZStack::new(
                    #body
                )
            )
        } else {
            quote!(#body)
        };

        tokens.extend(body);
    }
}

impl Debug for CarbideBodyFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CarbideBodyFunction")
            .field("return_type", &self.return_type.to_token_stream().to_string())
            .field("body", &self.body)
            .finish()
    }
}

impl Parse for CarbideBodyFunction {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fn_token = syn::token::Fn::parse(input)?;
        let ident = keyword(input, "body").map(|span| Ident::new("body", span))?;

        let _paren_content;
        let paren = if let Ok(paren) = parse_parens(input) {
            _paren_content = paren.content;
            paren.token
        } else {
            Paren::default()
        };

        let arrow = syn::token::RArrow::parse(input)?;
        let return_type = Type::parse(input)?;

        let body = CarbideBlock::parse(input)?;

        Ok(CarbideBodyFunction {
            _fn_token: fn_token,
            _ident: ident,
            _parenthesis: paren,
            _arrow: arrow,
            return_type,
            body
        })
    }
}

#[derive(Clone)]
pub enum CarbideStructField {
    Optional {
        attrs: Vec<Attribute>,
        l: Let,
        ident: Ident,
        c: Colon,
        t: Type,
        eq: syn::token::Eq,
        expr: Expr,
    },
    Required {
        attrs: Vec<Attribute>,
        l: Let,
        ident: Ident,
        c: Colon,
        t: Type,
    }
}

impl CarbideStructField {
    fn to_struct_field(&self) -> TokenStream {

        match self {
            CarbideStructField::Optional {  ident, t, .. } => {
                quote!(
                    #[state] pub #ident: #t
                )
            }
            CarbideStructField::Required {  ident, t, .. } => {
                quote!(
                    #[state] pub #ident: #t
                )
            }
        }
    }

    fn to_struct_builder_field(&self) -> TokenStream {

        match self {
            CarbideStructField::Optional {  ident, t, .. } => {
                quote!(
                    pub #ident: #t
                )
            }
            CarbideStructField::Required {  ident, t, .. } => {
                quote!(
                    pub #ident: #t
                )
            }
        }
    }

    fn to_struct_init_field(&self) -> TokenStream {
        match self {
            CarbideStructField::Required {  ident, .. } => {
                quote!(
                    #ident: #ident.into()
                )
            }
            CarbideStructField::Optional {  ident, expr, .. } => {
                quote!(
                    #ident: carbide_core::state::LocalState::new({
                        #expr
                    })
                )
            }
        }
    }

    fn to_struct_field_names(&self) -> TokenStream {
        match self {
            CarbideStructField::Required { ident, .. } => {
                quote!(
                    #ident
                )
            }
            CarbideStructField::Optional { ident, .. } => {
                quote!(
                    #ident
                )
            }
        }
    }

    fn to_optional_function(&self) -> Option<ItemFn> {
        match self {
            CarbideStructField::Optional { ident, t: ty, .. } => {

                let method_ident = Ident::new(&format!("with_optional_{}", ident.to_string()), ident.span());

                Some(parse_quote!(
                    pub fn #method_ident (mut self, #ident: impl Into<#ty>) -> Self {
                        self. #ident = #ident.into();
                        self
                    }
                ))
            }
            CarbideStructField::Required { .. } => None,
        }
    }

    fn to_required_arg(&self) -> Option<TokenStream> {
        match self {
            CarbideStructField::Required {  ident, t, .. } => {
                Some(quote!(
                        #ident: impl Into<#t>
                    ))
            }
            CarbideStructField::Optional { .. } => None,
        }
    }

    fn to_required_arg_name(&self) -> Option<Ident> {
        match self {
            CarbideStructField::Required { ident, .. } => {
                Some(ident.clone())
            }
            CarbideStructField::Optional { .. } => None,
        }
    }
}

impl Debug for CarbideStructField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CarbideStructField::Optional { ident, t, expr,  .. } => {
                f.debug_struct("Required")
                    .field("ident", &ident.to_string())
                    .field("type", &t.to_token_stream().to_string())
                    .field("expr", &expr.to_token_stream().to_string())
                    .finish()
            }
            CarbideStructField::Required { ident, t, .. } => {
                f.debug_struct("Optional")
                    .field("ident", &ident.to_string())
                    .field("type", &t.to_token_stream().to_string())
                    .finish()
            }
        }
    }
}

impl Parse for CarbideStructField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let l = Let::parse(input)?;
        let ident = Ident::parse(input)?;
        let col = Colon::parse(input)?;
        let types = Type::parse(input)?;

        Ok(if let Ok(token) = syn::token::Eq::parse(input) {
            let types = parse_quote!(carbide_core::state::TState<#types>);

            Optional {
                attrs,
                l,
                ident,
                c: col,
                t: types,
                eq: token,
                expr: Expr::parse(input)?
            }
        } else {
            let types = parse_quote!(carbide_core::state::TState<#types>);

            Required {
                attrs,
                l,
                ident,
                c: col,
                t: types
            }
        })
    }
}