use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::iter::FromIterator;
use proc_macro2::Ident;
use carbide_syn::{Error, Expr, ExprClosure, ExprUnary, parse_quote, UnOp};
use carbide_syn::fold::{Fold, fold_expr};
use carbide_syn::spanned::Spanned;

pub fn process_a_expr(ast: Expr) -> Expr {
    let mut collector = UnaryDollarIdentCollector(vec![]);

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

            let body: Expr = parse_quote!({
                #(
                    let mut #idents = Clone::clone(&#idents);
                    let mut #idents = carbide_core::state::State::value_mut(&mut #idents);
                )*

                #body
            });


            Expr::Closure(ExprClosure {
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
}