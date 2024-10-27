use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::iter::FromIterator;
use proc_macro2::{Ident, Span};
use quote::ToTokens;
use carbide_syn::{Error, Expr, ExprClosure, ExprUnary, parse_quote, UnOp, Macro, Token, parse_macro_input};
use carbide_syn::fold::{Fold, fold_expr};
use carbide_syn::parse::{Parse, ParseStream};
use carbide_syn::punctuated::Punctuated;
use carbide_syn::spanned::Spanned;
use carbide_syn::token::Token;
use crate::utils::get_crate_name;

pub fn process_a_expr(ast: Expr) -> Expr {
    let crate_name = get_crate_name();

    let mut collector = UnaryDollarIdentCollector(vec![]);
    //panic!("{:#?}", ast);

    let res = collector.fold_expr(ast);

    let deduplicated: HashSet<Ident, RandomState> = HashSet::from_iter(collector.0.into_iter());
    let idents = deduplicated.into_iter().collect::<Vec<_>>();

    //panic!("{:?}", idents);

    match res {
        Expr::Closure(
            ExprClosure {
                attrs,
                lifetimes,
                constness,
                movability,
                asyncness,
                capture: _,
                or1_token,
                inputs,
                or2_token,
                output,
                body
            }
        ) => {

            /*let new_idents = idents.iter().map(|a| {
                Ident::new(&format!("new_{}", a), Span::call_site())
            }).collect::<Vec<_>>();*/

            let body: Expr = parse_quote!({
                #(
                    let mut #idents = Clone::clone(&#idents);
                    let #idents = &mut *#crate_name::state::State::value_mut(&mut #idents);
                )*

                #body
            });

            let closure = Expr::Closure(ExprClosure {
                attrs,
                lifetimes,
                constness,
                movability,
                asyncness,
                capture: Some(parse_quote!(move)),
                or1_token,
                inputs,
                or2_token,
                output,
                body: Box::new(body),
            });

            parse_quote!({
                #(
                    let mut #idents = Clone::clone(&#idents);
                )*

                #closure
            })
        }
        e => Expr::Verbatim(Error::new(e.span(), "It is expected that the outermost expression is a closure").into_compile_error())
    }
}


pub(crate) struct UnaryDollarIdentCollector(Vec<Ident>);

impl Fold for UnaryDollarIdentCollector {
    fn fold_expr(&mut self, i: Expr) -> Expr {
        match i {
            Expr::Unary(ExprUnary { attrs: _, op: UnOp::Dollar(_), expr }) => {
                match *expr {
                    Expr::Path(path) if path.path.get_ident().is_some() => {
                        self.0.push(path.path.get_ident().unwrap().clone());
                        Expr::Path(path)
                    }
                    e => Expr::Verbatim(Error::new(e.span(), "The dollar operator must be followed by a state identifier").into_compile_error()),
                }
            }
            i => fold_expr(self, i),
        }
    }

    fn fold_macro(&mut self, i: Macro) -> Macro {
        let Macro {
            path,
            bang_token,
            delimiter,
            tokens
        } = i;

        let tokens = if let Ok(ExprList { expressions }) = carbide_syn::parse::<ExprList>(tokens.clone().into()) {
            let mut res = Punctuated::<Expr, Token![,]>::new();

            for expression in expressions {
                res.push(self.fold_expr(expression));
            }

            res.to_token_stream()
        } else {
            tokens
        };

        Macro {
            path,
            bang_token,
            delimiter,
            tokens,
        }
    }
}

struct ExprList {
    expressions: Punctuated<Expr, Token![,]>,
}

impl Parse for ExprList {
    fn parse(input: ParseStream) -> carbide_syn::Result<Self> {
        let expressions = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;
        Ok(ExprList { expressions })
    }
}