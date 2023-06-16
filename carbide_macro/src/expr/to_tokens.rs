use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Lit;
use crate::expr::carbide_expr::{BinaryExpr, CarbideBinOp, CarbideExpr, CarbideMember, CarbideUnOp, FieldExpr, IndexExpr, LitExpr, MacroExpr, MethodCallExpr, ParenExpr, PathExpr, StateExpr, UnaryExpr};

impl ToTokens for CarbideExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            CarbideExpr::Path(i) => {i.to_tokens(tokens)}
            CarbideExpr::Paren(i) => {i.to_tokens(tokens)}
            CarbideExpr::State(i) => {i.to_tokens(tokens)}
            CarbideExpr::Field(i) => {i.to_tokens(tokens)}
            CarbideExpr::Index(i) => {i.to_tokens(tokens)}
            CarbideExpr::Lit(i) => {i.to_tokens(tokens)}
            CarbideExpr::Macro(i) => {i.to_tokens(tokens)}
            CarbideExpr::MethodCall(i) => {i.to_tokens(tokens)}
            CarbideExpr::Unary(i) => {i.to_tokens(tokens)}
            CarbideExpr::Binary(i) => {i.to_tokens(tokens)}
        }
    }
}

impl ToTokens for MethodCallExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.receiver.to_tokens(tokens);
        self.dot_token.to_tokens(tokens);
        self.method.to_tokens(tokens);
        self.turbofish.to_tokens(tokens);
        self.paren_token.surround(tokens, |tokens| {
            self.args.to_tokens(tokens);
        });
    }
}

impl ToTokens for MacroExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.mac.to_tokens(tokens);
    }
}

impl ToTokens for LitExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let LitExpr {
            lit
        } = self;

        match lit {
            Lit::Str(_s) => {
                tokens.extend(quote!(
                    carbide_core::state::ValueState::new(#lit . to_string())
                ))
            }
            _ => {
                tokens.extend(quote!(
                    carbide_core::state::ValueState::new(#lit)
                ))
            }
        }


    }
}

impl ToTokens for PathExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        CarbideExpr::print_path(tokens, &self.qself, &self.path);
    }
}

impl ToTokens for StateExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let StateExpr {
            dollar_token: _,
            expr
        } = self;

        tokens.extend(quote!(
            #expr.clone()
        ))
    }
}

impl ToTokens for FieldExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let FieldExpr {
            base,
            dot_token: _,
            member
        } = self;

        tokens.extend(quote!(
            carbide_core::state::FieldState::new2(
                #base,
                |item| { &item.#member },
                |item| { &mut item.#member }
            )
        ))
    }
}

impl ToTokens for IndexExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let IndexExpr {
            expr,
            bracket_token: _,
            index
        } = self;

        tokens.extend(quote!(
            {
                carbide_core::state::IndexableState::index(&#expr, &carbide_core::state::TState::from(#index.clone()))
            }
        ))
    }
}

impl ToTokens for CarbideMember {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            CarbideMember::Named(ident) => {
                tokens.extend(quote!(
                    #ident
                ))
            }
            CarbideMember::Unnamed { index, .. } => {
                tokens.extend(quote!(
                    #index
                ))
            }
        }
    }
}

impl ToTokens for ParenExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.paren_token.surround(tokens, |tokens| {
            self.expr.to_tokens(tokens);
        });
    }
}

impl ToTokens for UnaryExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {

        let UnaryExpr {
            op,
            expr
        } = self;

        match op {
            CarbideUnOp::Not(_) => {
                tokens.extend(quote!(
                    !#expr
                ))
            }
            CarbideUnOp::Neg(_) => {
                tokens.extend(quote!(
                    -#expr
                ))
            }
        }
    }
}

impl ToTokens for BinaryExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {

        let BinaryExpr {
            left,
            op,
            right
        } = self;

        match op {
            // PartialEq
            CarbideBinOp::Eq(_) => {
                tokens.extend(quote!(
                    carbide_core::state::eq::StatePartialEq::eq({#left}, {#right})
                ))
            }
            CarbideBinOp::Ne(_) => {
                tokens.extend(quote!(
                    carbide_core::state::eq::StatePartialEq::ne({#left}, {#right})
                ))
            }

            // PartialOrd
            CarbideBinOp::Le(_) => {
                tokens.extend(quote!(
                    carbide_core::state::ord::StatePartialOrd::le({#left}, {#right})
                ))
            }
            CarbideBinOp::Lt(_) => {
                tokens.extend(quote!(
                    carbide_core::state::ord::StatePartialOrd::lt({#left}, {#right})
                ))
            }
            CarbideBinOp::Ge(_) => {
                tokens.extend(quote!(
                    carbide_core::state::ord::StatePartialOrd::ge({#left}, {#right})
                ))
            }
            CarbideBinOp::Gt(_) => {
                tokens.extend(quote!(
                    carbide_core::state::ord::StatePartialOrd::gt({#left}, {#right})
                ))
            }

            // Lazy boolean operators
            CarbideBinOp::And(_) => {
                tokens.extend(quote!(
                    carbide_core::state::and::StateAnd::and({#left}, {#right})
                ))
            }
            CarbideBinOp::Or(_) => {
                tokens.extend(quote!(
                    carbide_core::state::or::StateOr::or({#left}, {#right})
                ))
            }

            // Rest
            _ => {
                left.to_tokens(tokens);
                op.to_tokens(tokens);
                right.to_tokens(tokens);
            }
        }

    }
}

impl ToTokens for CarbideBinOp {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            CarbideBinOp::Add(t) => t.to_tokens(tokens),
            CarbideBinOp::Sub(t) => t.to_tokens(tokens),
            CarbideBinOp::Mul(t) => t.to_tokens(tokens),
            CarbideBinOp::Div(t) => t.to_tokens(tokens),
            CarbideBinOp::Rem(t) => t.to_tokens(tokens),
            CarbideBinOp::And(t) => t.to_tokens(tokens),
            CarbideBinOp::Or(t) => t.to_tokens(tokens),
            CarbideBinOp::BitXor(t) => t.to_tokens(tokens),
            CarbideBinOp::BitAnd(t) => t.to_tokens(tokens),
            CarbideBinOp::BitOr(t) => t.to_tokens(tokens),
            CarbideBinOp::Shl(t) => t.to_tokens(tokens),
            CarbideBinOp::Shr(t) => t.to_tokens(tokens),
            CarbideBinOp::Eq(t) => t.to_tokens(tokens),
            CarbideBinOp::Lt(t) => t.to_tokens(tokens),
            CarbideBinOp::Le(t) => t.to_tokens(tokens),
            CarbideBinOp::Ne(t) => t.to_tokens(tokens),
            CarbideBinOp::Ge(t) => t.to_tokens(tokens),
            CarbideBinOp::Gt(t) => t.to_tokens(tokens),
        }
    }
}