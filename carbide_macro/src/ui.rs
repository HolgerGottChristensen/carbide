use proc_macro2::Ident;
use carbide_syn::{Error, Expr, ExprMatch, parse_quote, PatIdent};
use carbide_syn::fold::{Fold, fold_expr};
use carbide_syn::spanned::Spanned;

pub fn ui(expr: Expr) -> Expr {

    match expr {
        Expr::ForLoop(_) | Expr::If(_) | Expr::Match(_) => (),
        _ => return Expr::Verbatim(Error::new(expr.span(), "Expected either for-loop, if expression or match expression").into_compile_error())
    }


    Folder {
        allow_widget_expr: true
    }.fold_expr(expr)
}

struct Folder {
    allow_widget_expr: bool,
}

impl Fold for Folder {
    fn fold_expr(&mut self, i: Expr) -> Expr {
        match i {
            Expr::Match(m) if self.allow_widget_expr => Self::handle_match_widget_expr(m),
            _ => fold_expr(self, i)
        }
    }
}

impl Folder {
    fn handle_match_widget_expr(m: ExprMatch) -> Expr {
        let ExprMatch {
            expr,
            arms,
            ..
        } = m;

        let expr = fold_expr(&mut Folder { allow_widget_expr: false }, *expr);

        let mut bindings = arms.iter().cloned().map(|a| {
            let mut folder = PatFolder(vec![]);
            folder.fold_pat(a.pat);
            folder.0
        }).collect::<Vec<_>>();

        let patterns = arms.iter().cloned().map(|a| a.pat).collect::<Vec<_>>();
        let mut patterns_rev = patterns.clone();

        let mut bodies = arms.iter().cloned().map(|a| {
            fold_expr(&mut Folder { allow_widget_expr: true }, *a.body)
        }).collect::<Vec<_>>();

        patterns_rev.reverse();
        bodies.reverse();
        bindings.reverse();

        parse_quote!({
            {
                if false {
                    // This is generated to let the compiler check for exhaustiveness
                    // This is expected to be optimized out by the compiler,
                    // because it is wrapped in a if false.
                    #[allow(unused_variables)]
                    match &*carbide::state::ReadState::value(&(#expr).clone()) {
                        #(
                            #patterns => unreachable!(),
                        )*
                    }
                }
            }

            // Might want to make an Unreachable Widget, since we check above that the match is exhaustive.
            let acc = carbide::widget::Empty::new();

            #(
                let acc = carbide::widget::IfElse::new(carbide::state::Map1::read_map((#expr).clone(), |a| {matches!(a, #patterns_rev)} ))
                    .when_true({
                        #(
                            let #bindings = carbide::state::FieldState::new((#expr).clone(), |a| {
                                match a {
                                    #patterns_rev => {
                                        #bindings
                                    }
                                    _ => panic!("Not matching: &{}", stringify!{#bindings})
                                }
                            }, |b| {
                                match b {
                                    #patterns_rev => {
                                        #bindings
                                    }
                                    _ => panic!("Not matching: &mut {}", stringify!{#bindings})
                                }
                            });
                        )*
                        #bodies
                    })
                    .when_false(acc);
            )*

            acc
        })
    }
}

struct PatFolder(Vec<Ident>);

impl Fold for PatFolder {
    fn fold_pat_ident(&mut self, i: PatIdent) -> PatIdent {
        if i.ident.to_string().chars().next().map_or(false, |c| c.is_lowercase()) {
            self.0.push(i.ident.clone());
        }

        i
    }
}
