use proc_macro2::Ident;
use carbide_syn::{Error, Expr, ExprMatch, parse_quote, PatIdent};
use carbide_syn::fold::{Fold, fold_expr};
use carbide_syn::spanned::Spanned;

pub fn ui(expr: Expr) -> Expr {

    match expr {
        Expr::ForLoop(_) | Expr::If(_) | Expr::Match(_) => (),
        _ => return Expr::Verbatim(Error::new(expr.span(), "Expected either for-loop, if expression or match expression").into_compile_error())
    }


    Folder {allow_widget_expr: true}.fold_expr(expr)
}

struct Folder {
    allow_widget_expr: bool,
}

impl Fold for Folder {
    fn fold_expr(&mut self, i: Expr) -> Expr {
        match i {
            Expr::Match(m) if self.allow_widget_expr => {
                let ExprMatch {
                    expr,
                    arms,
                    ..
                } = m;

                let expr = fold_expr(&mut Folder { allow_widget_expr: false }, *expr);

                let bindings = arms.iter().cloned().map(|a| {
                    let mut folder = PatFolder(vec![]);
                    folder.fold_pat(a.pat);
                    folder.0
                }).collect::<Vec<_>>();

                let patterns = arms.iter().cloned().map(|a| a.pat).collect::<Vec<_>>();
                let bodies = arms.iter().cloned().map(|a| a.body).collect::<Vec<_>>();

                parse_quote!({
                    {
                        if false {
                            // This is generated to let the compiler check for exhaustiveness
                            // This is expected to be optimized out by the compiler,
                            // because it is wrapped in a if false.
                            #[allow(unused_variables)]
                            match &*carbide::state::ReadState::value(&(#expr).clone()) {
                                #(
                                    #patterns => todo!(),
                                )*
                            }
                        }
                    }
                    #[allow(unused_variables)]
                    carbide::widget::Match::new((#expr).clone())
                    #(

                        .case((|a| matches!(a, #patterns), {
                            #(
                                let #bindings = carbide::state::FieldState::new((#expr).clone(), |a| {
                                    match a {
                                        #patterns => {
                                            #bindings
                                        }
                                        _ => panic!("Not matching: &{}", stringify!{#bindings})
                                    }
                                }, |b| {
                                    match b {
                                        #patterns => {
                                            #bindings
                                        }
                                        _ => panic!("Not matching: &mut {}", stringify!{#bindings})
                                    }
                                });
                            )*
                            #bodies
                        }))
                    )*
                })
            },
            _ => fold_expr(self, i)
        }
    }
}

struct PatFolder(Vec<Ident>);

impl Fold for PatFolder {
    fn fold_pat_ident(&mut self, i: PatIdent) -> PatIdent {
        self.0.push(i.ident.clone());

        i
    }
}
