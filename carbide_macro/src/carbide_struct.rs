use std::fmt::{Debug, Formatter};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::token::{Brace, Colon, Let, Paren, Struct, Token};
use syn::{AngleBracketedGenericArguments, Block, braced, Expr, Field, FnArg, GenericArgument, ItemFn, parenthesized, parse_quote, Pat, Path, PathArguments, PathSegment, PatIdent, PatType, Receiver, ReturnType, Signature, Token, TraitBound, TraitBoundModifier, Type, TypeParamBound, TypePath, TypeTraitObject, Visibility, VisPublic};
use syn::__private::parse_parens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::parsing::keyword;
use crate::carbide_expression::CarbideExpression;
use crate::carbide_struct::CarbideStructField::{Required, Optional};


pub struct CarbideStruct {
    struct_token: Token![struct],
    ident: Ident,
    brace: Brace,
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
            struct_token,
            ident,
            brace,
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

        let optional_field_methods = fields.iter().filter_map(|field| {
            field.to_optional_function()
        });

        let required_fields_args = fields.iter().filter_map(|field| {
            field.to_required_arg()
        });

        let struct_init_fields = fields.iter().map(|field| {
            field.to_struct_init_field()
        });

        // If we have a body with multiple widget returns we store them in a field in the struct
        let child_field = if self.body.body.len() == 0 {
            quote!()
        } else if self.body.body.len() == 1 {
            quote!(child: Box<dyn Widget>,)
        } else {
            quote!(child: Vec<Box<dyn Widget>>,)
        };

        let child_init_field = if self.body.body.len() == 0 {
            quote!()
        } else {
            quote!(child: children,)
        };

        let children_let = if self.body.body.len() == 0 {
            quote!()
        } else if self.body.body.len() == 1 {
            quote!(
                let children = #body;
            )
        } else {
            quote!(
                let children = vec![
                    #body
                ];
            )
        };

        let children_common = if self.body.body.len() == 0 {
            quote!()
        } else if self.body.body.len() == 1 {
            quote!(
                child: self.child,
            )
        } else {
            quote!(
                children: self.child,
            )
        };

        tokens.extend(quote!(
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
                    #children_let

                    Box::new(#ident {
                        id: WidgetId::new(),
                        position: Position::new(0.0, 0.0),
                        dimension: Dimension::new(100.0, 100.0),
                        #(#struct_init_fields, )*
                        #child_init_field
                    })
                }
            }

            /// Optional fields impl
            impl #ident {
                #(#optional_field_methods)*
            }

            carbide_core::CommonWidgetImpl!(#ident, self, id: self.id, #children_common position: self.position, dimension: self.dimension);

            impl WidgetExt for #ident {}
        ))
    }
}

pub struct CarbideBodyFunction {
    fn_token: Token![fn],
    ident: Ident,
    parenthesis: Paren,
    arrow: Token![->],
    return_type: Type,
    braces: Brace,
    body: Vec<CarbideExpression>,
}

impl ToTokens for CarbideBodyFunction {
    fn to_tokens(&self, tokens: &mut TokenStream) {

        let CarbideBodyFunction {
            body,
            ..
        } = self;

        tokens.extend(quote!(
            #(#body),*
        ))
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

        let paren_content;
        let paren = if let Ok(paren) = parse_parens(input) {
            paren_content = paren.content;
            paren.token
        } else {
            Paren::default()
        };

        let arrow = syn::token::RArrow::parse(input)?;
        let return_type = Type::parse(input)?;

        let brace_content;
        let brace = braced!(brace_content in input);

        let mut body = vec![];

        while let Ok(expr) = CarbideExpression::parse(&brace_content) {
            body.push(expr);
        }

        Ok(CarbideBodyFunction {
            fn_token,
            ident,
            parenthesis: paren,
            arrow,
            return_type,
            braces: brace,
            body
        })
    }
}

pub enum CarbideStructField {
    Optional {
        l: Let,
        ident: Ident,
        c: Colon,
        t: Type,
        eq: syn::token::Eq,
        expr: Expr,
    },
    Required {
        l: Let,
        ident: Ident,
        c: Colon,
        t: Type,
    }
}

impl CarbideStructField {
    fn to_struct_field(&self) -> Field {
        Field {
            attrs: vec![],
            vis: Visibility::Public(VisPublic { pub_token: Default::default() }),
            ident: Some(match self {
                CarbideStructField::Optional { ident, .. } => ident.clone(),
                CarbideStructField::Required { ident, .. } => ident.clone(),
            }),
            colon_token: Some(Colon::default()),
            ty: match self {
                CarbideStructField::Optional { t, .. } => t.clone(),
                CarbideStructField::Required { t, .. } => t.clone(),
            }
        }
    }

    fn to_struct_init_field(&self) -> TokenStream {
        match self {
            CarbideStructField::Required { ident, .. } => {
                quote!(
                    #ident
                )
            }
            CarbideStructField::Optional { ident, expr, .. } => {
                quote!(
                    #ident: {
                        #expr
                    }
                )
            }
        }
    }

    fn to_optional_function(&self) -> Option<ItemFn> {
        match self {
            CarbideStructField::Optional { ident, t, .. } => {

                // Make a punctuated list of parameters for the optional function
                let mut inputs = Punctuated::new();

                // Push "mut self" to the list of parameters
                inputs.push(FnArg::Receiver(Receiver {
                    attrs: vec![],
                    reference: None,
                    mutability: Some(Default::default()),
                    self_token: Default::default()
                }));

                // Push "ident: type" to the list of parameters
                inputs.push(FnArg::Typed(PatType {
                    attrs: vec![],
                    pat: Box::new(Pat::Ident(PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        ident: ident.clone(),
                        subpat: None
                    })),
                    colon_token: Default::default(),
                    ty: Box::new(t.clone())
                }));

                Some(ItemFn {
                    attrs: vec![],
                    vis: Visibility::Public(VisPublic {pub_token: Default::default() }),
                    sig: Signature {
                        constness: None,
                        asyncness: None,
                        unsafety: None,
                        abi: None,
                        fn_token: Default::default(),
                        ident: Ident::new(&format!("with_optional_{}", ident.to_string()), Span::call_site()),
                        generics: Default::default(),
                        paren_token: Default::default(),
                        inputs,
                        variadic: None,
                        output: parse_quote!(-> Box<Self>),
                    },
                    block: parse_quote!({
                        self.#ident = #ident;
                        Box::new(self)
                    })
                })
            }
            CarbideStructField::Required { .. } => None,
        }
    }

    fn to_required_arg(&self) -> Option<FnArg> {
        match self {
            CarbideStructField::Required { ident, t, .. } => {
                Some(FnArg::Typed(PatType {
                    attrs: vec![],
                    pat: Box::new(Pat::Ident(PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        ident: ident.clone(),
                        subpat: None
                    })),
                    colon_token: Default::default(),
                    ty: Box::new(t.clone())
                }))
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
        let l = Let::parse(input)?;
        let ident = Ident::parse(input)?;
        let col = Colon::parse(input)?;
        let types = Type::parse(input)?;

        Ok(if let Ok(token) = syn::token::Eq::parse(input) {
            Optional {
                l,
                ident,
                c: col,
                t: types,
                eq: token,
                expr: Expr::parse(input)?
            }
        } else {
            Required {
                l,
                ident,
                c: col,
                t: types
            }
        })
    }
}