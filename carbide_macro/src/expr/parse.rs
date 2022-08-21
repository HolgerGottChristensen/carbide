use syn::{ExprPath, LitInt, parenthesized, Token};
use syn::parse::{Parse, ParseStream};
use crate::expr::carbide_expr::{CarbideBinOp, CarbideExpr, CarbideMember, CarbidePrecedence, CarbideUnOp, LitExpr, MacroExpr, ParenExpr, PathExpr, UnaryExpr};

impl Parse for PathExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expr = ExprPath::parse(input)?;
        Ok(PathExpr {
            qself: expr.qself,
            path: expr.path
        })
    }
}

impl Parse for CarbideMember {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Ident) {
            input.parse().map(CarbideMember::Named)
        } else if input.peek(LitInt) {
            let lit: LitInt = input.parse()?;
            if lit.suffix().is_empty() {
                Ok(CarbideMember::Unnamed {
                    index: lit
                        .base10_digits()
                        .parse()
                        .map_err(|err| syn::Error::new(lit.span(), err))?,
                    span: lit.span(),
                })
            } else {
                Err(syn::Error::new(lit.span(), "expected unsuffixed integer"))
            }
        } else {
            Err(input.error("expected identifier or integer"))
        }
    }
}

impl Parse for MacroExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(MacroExpr {
            mac: input.parse()?,
        })
    }
}

impl Parse for LitExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(LitExpr {
            lit: input.parse()?,
        })
    }
}

impl Parse for CarbideExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lhs = CarbideExpr::parse_unary_expr(input)?;
        CarbideExpr::parse_expr(input, lhs, CarbidePrecedence::Any)
    }
}

impl Parse for ParenExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let paren_token = parenthesized!(content in input);

        let inner = CarbideExpr::parse(&content)?;
        Ok(ParenExpr {
            paren_token,
            expr: Box::new(inner)
        })
    }
}

impl Parse for UnaryExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(UnaryExpr {
            op: input.parse()?,
            expr: Box::new(CarbideExpr::parse_unary_expr(input)?),
        })
    }
}

impl Parse for CarbideUnOp {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![!]) {
            input.parse().map(CarbideUnOp::Not)
        } else if lookahead.peek(Token![-]) {
            input.parse().map(CarbideUnOp::Neg)
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for CarbideBinOp {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![&&]) {
            input.parse().map(CarbideBinOp::And)
        } else if input.peek(Token![||]) {
            input.parse().map(CarbideBinOp::Or)
        } else if input.peek(Token![<<]) {
            input.parse().map(CarbideBinOp::Shl)
        } else if input.peek(Token![>>]) {
            input.parse().map(CarbideBinOp::Shr)
        } else if input.peek(Token![==]) {
            input.parse().map(CarbideBinOp::Eq)
        } else if input.peek(Token![<=]) {
            input.parse().map(CarbideBinOp::Le)
        } else if input.peek(Token![!=]) {
            input.parse().map(CarbideBinOp::Ne)
        } else if input.peek(Token![>=]) {
            input.parse().map(CarbideBinOp::Ge)
        } else if input.peek(Token![+]) {
            input.parse().map(CarbideBinOp::Add)
        } else if input.peek(Token![-]) {
            input.parse().map(CarbideBinOp::Sub)
        } else if input.peek(Token![*]) {
            input.parse().map(CarbideBinOp::Mul)
        } else if input.peek(Token![/]) {
            input.parse().map(CarbideBinOp::Div)
        } else if input.peek(Token![%]) {
            input.parse().map(CarbideBinOp::Rem)
        } else if input.peek(Token![^]) {
            input.parse().map(CarbideBinOp::BitXor)
        } else if input.peek(Token![&]) {
            input.parse().map(CarbideBinOp::BitAnd)
        } else if input.peek(Token![|]) {
            input.parse().map(CarbideBinOp::BitOr)
        } else if input.peek(Token![<]) {
            input.parse().map(CarbideBinOp::Lt)
        } else if input.peek(Token![>]) {
            input.parse().map(CarbideBinOp::Gt)
        } else {
            Err(input.error("expected binary operator"))
        }
    }
}