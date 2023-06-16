use std::cmp::Ordering;
use std::mem;
use proc_macro2::{Delimiter, Ident, Span, TokenStream, TokenTree};
use quote::{ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{bracketed, Index, Lit, LitFloat, Macro, MacroDelimiter, MethodTurbofish, parenthesized, Path, PathArguments, QSelf, Token};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket, Dollar, Paren};

#[derive(PartialEq, Clone, Debug)]
pub enum CarbideExpr {
    Lit(LitExpr),
    Binary(BinaryExpr),
    Path(PathExpr),
    Paren(ParenExpr),
    State(StateExpr),
    Unary(UnaryExpr),
    Field(FieldExpr),
    Index(IndexExpr),
    Macro(MacroExpr),
    MethodCall(MethodCallExpr),
}

#[derive(PartialEq, Clone, Debug)]
pub struct BinaryExpr {
    pub left: Box<CarbideExpr>,
    pub op: CarbideBinOp,
    pub right: Box<CarbideExpr>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct UnaryExpr {
    pub op: CarbideUnOp,
    pub expr: Box<CarbideExpr>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct ParenExpr {
    pub paren_token: Paren,
    pub expr: Box<CarbideExpr>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct MethodCallExpr {
    pub receiver: Box<CarbideExpr>,
    pub dot_token: Token![.],
    pub method: Ident,
    pub turbofish: Option<MethodTurbofish>,
    pub paren_token: Paren,
    pub args: Punctuated<CarbideExpr, Token![,]>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct MacroExpr {
    pub mac: Macro,
}

#[derive(PartialEq, Clone, Debug)]
pub struct LitExpr {
    pub lit: Lit,
}

#[derive(PartialEq, Clone, Debug)]
pub struct PathExpr {
    pub qself: Option<QSelf>,
    pub path: Path,
}

#[derive(PartialEq, Clone, Debug)]
pub struct StateExpr {
    pub dollar_token: Token![$],
    pub expr: Box<CarbideExpr>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct FieldExpr {
    pub base: Box<CarbideExpr>,
    pub dot_token: Token![.],
    pub member: CarbideMember,
}

#[derive(PartialEq, Clone, Debug)]
pub struct IndexExpr {
    pub expr: Box<CarbideExpr>,
    pub bracket_token: Bracket,
    pub index: Box<CarbideExpr>,
}

#[derive(Clone, Debug)]
pub enum CarbideMember {
    /// A named field like `self.x`.
    Named(Ident),
    /// An unnamed field like `self.0`.
    Unnamed {
        index: u32,
        span: Span,
    },
}

impl PartialEq for CarbideMember {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CarbideMember::Named(i), CarbideMember::Named(other_i)) => {
                i == other_i
            }
            (CarbideMember::Unnamed { index, .. }, CarbideMember::Unnamed { index: other_index, .. }) => {
                index == other_index
            }
            _ => false
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum CarbideUnOp {
    /// The `!` operator for logical inversion
    Not(Token![!]),
    /// The `-` operator for negation
    Neg(Token![-]),
}

#[derive(PartialEq, Clone, Debug)]
pub enum CarbideBinOp {
    /// The `+` operator (addition)
    Add(Token![+]),
    /// The `-` operator (subtraction)
    Sub(Token![-]),
    /// The `*` operator (multiplication)
    Mul(Token![*]),
    /// The `/` operator (division)
    Div(Token![/]),
    /// The `%` operator (modulus)
    Rem(Token![%]),
    /// The `&&` operator (logical and)
    And(Token![&&]),
    /// The `||` operator (logical or)
    Or(Token![||]),
    /// The `^` operator (bitwise xor)
    BitXor(Token![^]),
    /// The `&` operator (bitwise and)
    BitAnd(Token![&]),
    /// The `|` operator (bitwise or)
    BitOr(Token![|]),
    /// The `<<` operator (shift left)
    Shl(Token![<<]),
    /// The `>>` operator (shift right)
    Shr(Token![>>]),
    /// The `==` operator (equality)
    Eq(Token![==]),
    /// The `<` operator (less than)
    Lt(Token![<]),
    /// The `<=` operator (less than or equal to)
    Le(Token![<=]),
    /// The `!=` operator (not equal to)
    Ne(Token![!=]),
    /// The `>=` operator (greater than or equal to)
    Ge(Token![>=]),
    /// The `>` operator (greater than)
    Gt(Token![>]),
}

pub(crate) enum CarbidePrecedence {
    Any,
    Range,
    Or,
    And,
    Compare,
    BitOr,
    BitXor,
    BitAnd,
    Shift,
    Arithmetic,
    Term,
    Cast,
}

impl CarbidePrecedence {
    fn of(op: &CarbideBinOp) -> Self {
        match *op {
            CarbideBinOp::Add(_) | CarbideBinOp::Sub(_) => CarbidePrecedence::Arithmetic,
            CarbideBinOp::Mul(_) | CarbideBinOp::Div(_) | CarbideBinOp::Rem(_) => CarbidePrecedence::Term,
            CarbideBinOp::And(_) => CarbidePrecedence::And,
            CarbideBinOp::Or(_) => CarbidePrecedence::Or,
            CarbideBinOp::BitXor(_) => CarbidePrecedence::BitXor,
            CarbideBinOp::BitAnd(_) => CarbidePrecedence::BitAnd,
            CarbideBinOp::BitOr(_) => CarbidePrecedence::BitOr,
            CarbideBinOp::Shl(_) | CarbideBinOp::Shr(_) => CarbidePrecedence::Shift,
            CarbideBinOp::Eq(_)
            | CarbideBinOp::Lt(_)
            | CarbideBinOp::Le(_)
            | CarbideBinOp::Ne(_)
            | CarbideBinOp::Ge(_)
            | CarbideBinOp::Gt(_) => CarbidePrecedence::Compare,
        }
    }
}

impl Copy for CarbidePrecedence {}

impl Clone for CarbidePrecedence {
    fn clone(&self) -> Self {
        *self
    }
}

impl PartialEq for CarbidePrecedence {
    fn eq(&self, other: &Self) -> bool {
        *self as u8 == *other as u8
    }
}

impl PartialOrd for CarbidePrecedence {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let this = *self as u8;
        let other = *other as u8;
        Some(this.cmp(&other))
    }
}



/// This has been adapted from syn's src/expr.rs
impl CarbideExpr {
    pub(crate) fn parse_expr(input: ParseStream, mut lhs: CarbideExpr, base: CarbidePrecedence,) -> syn::Result<CarbideExpr> {
        loop {
            if input
                .fork()
                .parse::<CarbideBinOp>()
                .ok()
                .map_or(false, |op| CarbidePrecedence::of(&op) >= base)
            {
                let op = CarbideBinOp::parse(input)?;
                let precedence = CarbidePrecedence::of(&op);
                let mut rhs = CarbideExpr::parse_unary_expr(input)?;

                loop {
                    let next = CarbideExpr::peek_precedence(input);
                    if next > precedence {
                        rhs = CarbideExpr::parse_expr(input, rhs, next)?;
                    } else {
                        break;
                    }
                }

                lhs = CarbideExpr::Binary(BinaryExpr {
                    left: Box::new(lhs),
                    op,
                    right: Box::new(rhs),
                });
            } /*else if Precedence::Range >= base && input.peek(Token![..]) {
                let limits: RangeLimits = input.parse()?;
                let rhs = if input.is_empty()
                    || input.peek(Token![,])
                    || input.peek(Token![;])
                    || input.peek(Token![.]) && !input.peek(Token![..])
                    || !allow_struct.0 && input.peek(token::Brace)
                {
                    None
                } else {
                    let mut rhs = unary_expr(input, allow_struct)?;
                    loop {
                        let next = peek_precedence(input);
                        if next > Precedence::Range {
                            rhs = parse_expr(input, rhs, allow_struct, next)?;
                        } else {
                            break;
                        }
                    }
                    Some(rhs)
                };
                lhs = Expr::Range(ExprRange {
                    attrs: Vec::new(),
                    from: Some(Box::new(lhs)),
                    limits,
                    to: rhs.map(Box::new),
                });
            } else if Precedence::Cast >= base && input.peek(Token![as]) {
                let as_token: Token![as] = input.parse()?;
                let ty = input.call(Type::without_plus)?;
                check_cast(input)?;
                lhs = Expr::Cast(ExprCast {
                    attrs: Vec::new(),
                    expr: Box::new(lhs),
                    as_token,
                    ty: Box::new(ty),
                });
            } else if Precedence::Cast >= base && input.peek(Token![:]) && !input.peek(Token![::]) {
                let colon_token: Token![:] = input.parse()?;
                let ty = input.call(Type::without_plus)?;
                check_cast(input)?;
                lhs = Expr::Type(ExprType {
                    attrs: Vec::new(),
                    expr: Box::new(lhs),
                    colon_token,
                    ty: Box::new(ty),
                });
            }*/ else {
                break;
            }
        }
        Ok(lhs)
    }

    /// ! <trailer>
    /// - <trailer>
    pub(crate) fn parse_unary_expr(input: ParseStream) -> syn::Result<CarbideExpr> {

        if input.peek(Token![!]) || input.peek(Token!(-)) {
            UnaryExpr::parse(input).map(CarbideExpr::Unary)
        } else {
            CarbideExpr::parse_trailer_expr(input)
        }

    }

    /// <atom> . <ident> ...
    /// <atom> [ <expr> ] ...
    fn parse_trailer_expr(
        input: ParseStream,
    ) -> syn::Result<CarbideExpr> {
        let mut e = CarbideExpr::parse_atom_expr(input)?;

        loop {
            if input.peek(Token![.]) && !input.peek(Token![..])
            {
                let mut dot_token: Token![.] = input.parse()?;

                let float_token: Option<LitFloat> = input.parse()?;
                if let Some(float_token) = float_token {
                    if CarbideExpr::multi_index(&mut e, &mut dot_token, float_token)? {
                        continue;
                    }
                }

                let member = CarbideMember::parse(input)?;
                let turbofish = if matches!(member, CarbideMember::Named(_)) && input.peek(Token![::]) {
                    Some(input.parse::<MethodTurbofish>()?)
                } else {
                    None
                };

                if turbofish.is_some() || input.peek(Paren) {
                    if let CarbideMember::Named(method) = member {
                        let content;
                        e = CarbideExpr::MethodCall(MethodCallExpr {
                            receiver: Box::new(e),
                            dot_token,
                            method,
                            turbofish,
                            paren_token: parenthesized!(content in input),
                            args: content.parse_terminated(CarbideExpr::parse)?,
                        });
                        continue;
                    }
                }

                e = CarbideExpr::Field(FieldExpr {
                    base: Box::new(e),
                    dot_token,
                    member,
                });
            } else if input.peek(Bracket) {
                let content;
                e = CarbideExpr::Index(IndexExpr {
                    expr: Box::new(e),
                    bracket_token: bracketed!(content in input),
                    index: content.parse()?,
                });
            } else {
                break;
            }
        }

        Ok(e)
    }

    fn parse_atom_expr(input: ParseStream) -> syn::Result<CarbideExpr> {
        if input.peek(Lit) {
            input.parse().map(CarbideExpr::Lit)
        } else if input.peek(syn::Ident)
            || input.peek(Token![::])
            || input.peek(Token![<])
            || input.peek(Token![self])
            || input.peek(Token![Self])
            || input.peek(Token![super])
            || input.peek(Token![crate])
            || input.peek(Token![$])
        {
            CarbideExpr::parse_path_or_macro(input)
        } else if input.peek(Paren) {
            input.parse().map(CarbideExpr::Paren)
        } else {
            Err(input.error("unsupported expression"))
        }
    }

    fn parse_path_or_macro(input: ParseStream) -> syn::Result<CarbideExpr> {

        let state = Dollar::parse(input).ok();

        let expr: PathExpr = input.parse()?;

        if expr.qself.is_none() && state.is_none() && input.peek(Token![!]) && !input.peek(Token![!=]) {
            let mut contains_arguments = false;
            for segment in &expr.path.segments {
                match segment.arguments {
                    PathArguments::None => {}
                    PathArguments::AngleBracketed(_) | PathArguments::Parenthesized(_) => {
                        contains_arguments = true;
                    }
                }
            }

            if !contains_arguments {
                let bang_token: Token![!] = input.parse()?;
                let (delimiter, tokens) = CarbideExpr::parse_macro_delimiter(input)?;
                return Ok(CarbideExpr::Macro(MacroExpr {
                    mac: Macro {
                        path: expr.path,
                        bang_token,
                        delimiter,
                        tokens,
                    },
                }));
            }
        }

        if let Some(dollar) = state {
            Ok(CarbideExpr::State(StateExpr {
                dollar_token: dollar,
                expr: Box::new(CarbideExpr::Path(expr))
            }))
        } else {
            Ok(CarbideExpr::Path(expr))
        }


    }

    fn peek_precedence(input: ParseStream) -> CarbidePrecedence {
        if let Ok(op) = input.fork().parse() {
            CarbidePrecedence::of(&op)
        } else if input.peek(Token![..]) {
            CarbidePrecedence::Range
        } else if input.peek(Token![as]) || input.peek(Token![:]) && !input.peek(Token![::]) {
            CarbidePrecedence::Cast
        } else {
            CarbidePrecedence::Any
        }
    }
}


/// Helper functions
impl CarbideExpr {
    fn dummy() -> CarbideExpr {
        CarbideExpr::Path(PathExpr {
            qself: None,
            path: Path { leading_colon: None, segments: Default::default() }
        })
    }

    fn multi_index(e: &mut CarbideExpr, dot_token: &mut Token![.], float: LitFloat) -> syn::Result<bool> {
        let mut float_repr = float.to_string();
        let trailing_dot = float_repr.ends_with('.');
        if trailing_dot {
            float_repr.truncate(float_repr.len() - 1);
        }
        for part in float_repr.split('.') {
            let index: Index = syn::parse_str(part).map_err(|err| syn::Error::new(float.span(), err))?;

            let base = mem::replace(e, CarbideExpr::dummy());

            *e = CarbideExpr::Field(FieldExpr {
                base: Box::new(base),
                dot_token: Token![.](dot_token.span),
                member: CarbideMember::Unnamed {
                    index: index.index,
                    span: index.span
                },
            });
            *dot_token = Token![.](float.span());
        }
        Ok(!trailing_dot)
    }

    pub(crate) fn print_path(tokens: &mut TokenStream, qself: &Option<QSelf>, path: &Path) {
        pub struct TokensOrDefault<'a, T: 'a>(pub &'a Option<T>);

        impl<'a, T> ToTokens for TokensOrDefault<'a, T> where T: ToTokens + Default {
            fn to_tokens(&self, tokens: &mut TokenStream) {
                match self.0 {
                    Some(t) => t.to_tokens(tokens),
                    None => T::default().to_tokens(tokens),
                }
            }
        }

        let qself = match qself {
            Some(qself) => qself,
            None => {
                path.to_tokens(tokens);
                return;
            }
        };
        qself.lt_token.to_tokens(tokens);
        qself.ty.to_tokens(tokens);

        let pos = std::cmp::min(qself.position, path.segments.len());
        let mut segments = path.segments.pairs();
        if pos > 0 {
            TokensOrDefault(&qself.as_token).to_tokens(tokens);
            path.leading_colon.to_tokens(tokens);
            for (i, segment) in segments.by_ref().take(pos).enumerate() {
                if i + 1 == pos {
                    segment.value().to_tokens(tokens);
                    qself.gt_token.to_tokens(tokens);
                    segment.punct().to_tokens(tokens);
                } else {
                    segment.to_tokens(tokens);
                }
            }
        } else {
            qself.gt_token.to_tokens(tokens);
            path.leading_colon.to_tokens(tokens);
        }
        for segment in segments {
            segment.to_tokens(tokens);
        }
    }

    pub fn parse_macro_delimiter(input: ParseStream) -> syn::Result<(MacroDelimiter, TokenStream)> {
        input.step(|cursor| {
            if let Some((TokenTree::Group(g), rest)) = cursor.token_tree() {
                let span = g.span();
                let delimiter = match g.delimiter() {
                    Delimiter::Parenthesis => MacroDelimiter::Paren(Paren(span)),
                    Delimiter::Brace => MacroDelimiter::Brace(Brace(span)),
                    Delimiter::Bracket => MacroDelimiter::Bracket(Bracket(span)),
                    Delimiter::None => {
                        return Err(cursor.error("expected delimiter"));
                    }
                };
                Ok(((delimiter, g.stream()), rest))
            } else {
                Err(cursor.error("expected delimiter"))
            }
        })
    }
}

mod tests {
    mod parse_tests {
        
        
        
        
        
        

        #[test]
        fn parse_lit() {
            // Arrange

            let stream = quote!(
                1
            );

            println!("{:#?}", &stream);

            // Act
            let actual: CarbideExpr = syn::parse2(stream).unwrap();

            // Assert
            assert!(matches!(actual, CarbideExpr::Lit(LitExpr {lit: Lit::Int(_)})))
        }

        #[test]
        fn parse_ident() {
            // Arrange

            let stream = quote!(
                test
            );

            println!("{:#?}", &stream);

            // Act
            let actual: CarbideExpr = syn::parse2(stream).unwrap();

            // Assert
            let expected = CarbideExpr::Path(PathExpr {
                qself: None,
                path: parse_quote!(test)
            });

            assert_eq!(expected, actual)
        }

        #[test]
        fn parse_unary1() {
            // Arrange

            let stream = quote!(
                !test
            );

            println!("{:#?}", &stream);

            // Act
            let actual: CarbideExpr = syn::parse2(stream).unwrap();

            // Assert
            let expected = CarbideExpr::Unary(UnaryExpr {
                op: CarbideUnOp::Not(Default::default()),
                expr: Box::new(CarbideExpr::Path(PathExpr {
                    qself: None,
                    path: parse_quote!(test)
                }))
            });

            assert_eq!(expected, actual)
        }

        #[test]
        fn parse_unary2() {
            // Arrange

            let stream = quote!(
                -!test
            );

            println!("{:#?}", &stream);

            // Act
            let actual: CarbideExpr = syn::parse2(stream).unwrap();

            // Assert
            let expected = CarbideExpr::Unary(UnaryExpr {
                op: CarbideUnOp::Neg(Default::default()),
                expr: Box::new(CarbideExpr::Unary(UnaryExpr {
                    op: CarbideUnOp::Not(Default::default()),
                    expr: Box::new(CarbideExpr::Path(PathExpr {
                        qself: None,
                        path: parse_quote!(test)
                    }))
                }))
            });

            assert_eq!(expected, actual)
        }

        #[test]
        fn parse_unary3() {
            // Arrange

            let stream = quote!(
                -!$test
            );

            println!("{:#?}", &stream);

            // Act
            let actual: CarbideExpr = syn::parse2(stream).unwrap();

            // Assert
            let expected = CarbideExpr::Unary(UnaryExpr {
                op: CarbideUnOp::Neg(Default::default()),
                expr: Box::new(CarbideExpr::Unary(UnaryExpr {
                    op: CarbideUnOp::Not(Default::default()),
                    expr: Box::new(CarbideExpr::State(StateExpr {
                        dollar_token: Default::default(),
                        expr: Box::new(CarbideExpr::Path(PathExpr {
                            qself: None,
                            path: parse_quote!(test)
                        }))
                    }))
                }))
            });

            assert_eq!(expected, actual)
        }

        #[test]
        fn parse_binary1() {
            // Arrange

            let stream = quote!(
                test1 == test2
            );

            println!("{:#?}", &stream);

            // Act
            let actual: CarbideExpr = syn::parse2(stream).unwrap();

            // Assert
            let expected = CarbideExpr::Binary(BinaryExpr {
                left: Box::new(parse_quote!(test1)),
                op: CarbideBinOp::Eq(Default::default()),
                right: Box::new(parse_quote!(test2))
            });

            assert_eq!(expected, actual)
        }

        #[test]
        fn parse_binary2() {
            // Arrange

            let stream = quote!(
                test1 == test2 == test3
            );

            println!("{:#?}", &stream);

            // Act
            let actual: CarbideExpr = syn::parse2(stream).unwrap();

            // Assert
            let expected = CarbideExpr::Binary(BinaryExpr {
                left: Box::new(CarbideExpr::Binary(BinaryExpr {
                    left: Box::new(parse_quote!(test1)),
                    op: CarbideBinOp::Eq(Default::default()),
                    right: Box::new(parse_quote!(test2))
                })),
                op: CarbideBinOp::Eq(Default::default()),
                right: Box::new(parse_quote!(test3)),
            });

            assert_eq!(expected, actual)
        }

        #[test]
        fn parse_binary3() {
            // Arrange

            let stream = quote!(
                test1 || test2 && test3
            );

            println!("{:#?}", &stream);

            // Act
            let actual: CarbideExpr = syn::parse2(stream).unwrap();

            // Assert
            let expected = CarbideExpr::Binary(BinaryExpr {
                left: Box::new(parse_quote!(test1)),
                op: CarbideBinOp::Or(Default::default()),
                right: Box::new(CarbideExpr::Binary(BinaryExpr {
                    left: Box::new(parse_quote!(test2)),
                    op: CarbideBinOp::And(Default::default()),
                    right: Box::new(parse_quote!(test3))
                })),
            });

            assert_eq!(expected, actual)
        }

        #[test]
        fn parse_binary4() {
            // Arrange

            let stream = quote!(
                test1 && test2 <= test3
            );

            println!("{:#?}", &stream);

            // Act
            let actual: CarbideExpr = syn::parse2(stream).unwrap();

            // Assert
            let expected = CarbideExpr::Binary(BinaryExpr {
                left: Box::new(parse_quote!(test1)),
                op: CarbideBinOp::And(Default::default()),
                right: Box::new(CarbideExpr::Binary(BinaryExpr {
                    left: Box::new(parse_quote!(test2)),
                    op: CarbideBinOp::Le(Default::default()),
                    right: Box::new(parse_quote!(test3))
                })),
            });

            assert_eq!(expected, actual)
        }

        #[test]
        fn parse_binary5() {
            // Arrange

            let stream = quote!(
                test1 + test2 * test3
            );

            println!("{:#?}", &stream);

            // Act
            let actual: CarbideExpr = syn::parse2(stream).unwrap();

            // Assert
            let expected = CarbideExpr::Binary(BinaryExpr {
                left: Box::new(parse_quote!(test1)),
                op: CarbideBinOp::Add(Default::default()),
                right: Box::new(CarbideExpr::Binary(BinaryExpr {
                    left: Box::new(parse_quote!(test2)),
                    op: CarbideBinOp::Mul(Default::default()),
                    right: Box::new(parse_quote!(test3))
                })),
            });

            assert_eq!(expected, actual)
        }

        #[test]
        fn parse_binary6() {
            // Arrange

            let stream = quote!(
                (test1 + test2) * test3
            );

            println!("{:#?}", &stream);

            // Act
            let actual: CarbideExpr = syn::parse2(stream).unwrap();

            // Assert
            let expected = CarbideExpr::Binary(BinaryExpr {
                left: Box::new(CarbideExpr::Paren(ParenExpr {
                    paren_token: Default::default(),
                    expr: Box::new(CarbideExpr::Binary(BinaryExpr {
                        left: Box::new(parse_quote!(test1)),
                        op: CarbideBinOp::Add(Default::default()),
                        right: Box::new(parse_quote!(test2))
                    }))
                })),
                op: CarbideBinOp::Mul(Default::default()),
                right: Box::new(parse_quote!(test3))
            });

            assert_eq!(expected, actual)
        }

        #[test]
        fn parse_parenthesis() {
            // Arrange

            let stream = quote!(
                (test)
            );

            println!("{:#?}", &stream);

            // Act
            let actual: CarbideExpr = syn::parse2(stream).unwrap();

            // Assert
            let expected = CarbideExpr::Paren(ParenExpr {
                paren_token: Default::default(),
                expr: Box::new(CarbideExpr::Path(PathExpr {
                    qself: None,
                    path: parse_quote!(test)
                }))
            });

            assert_eq!(expected, actual)
        }

        #[test]
        fn parse_state() {
            // Arrange

            let stream = quote!(
                $test
            );

            println!("{:#?}", &stream);

            // Act
            let actual: CarbideExpr = syn::parse2(stream).unwrap();

            // Assert
            let expected = CarbideExpr::State(StateExpr {
                dollar_token: Default::default(),
                expr: Box::new(CarbideExpr::Path(PathExpr {
                    qself: None,
                    path: parse_quote!(test)
                }))
            });

            assert_eq!(expected, actual)
        }

        #[test]
        fn parse_field1() {
            // Arrange

            let stream = quote!(
                test.field1
            );

            println!("{:#?}", &stream);

            // Act
            let actual: CarbideExpr = syn::parse2(stream).unwrap();

            // Assert
            let expected = CarbideExpr::Field(FieldExpr {
                base: Box::new(CarbideExpr::Path(PathExpr {
                    qself: None,
                    path: parse_quote!(test)
                })),
                dot_token: Default::default(),
                member: CarbideMember::Named(Ident::new("field1", Span::call_site()))
            });

            assert_eq!(expected, actual)
        }

        #[test]
        fn parse_field2() {
            // Arrange

            let stream = quote!(
                test.field1.field2
            );

            println!("{:#?}", &stream);

            // Act
            let actual: CarbideExpr = syn::parse2(stream).unwrap();

            // Assert
            let expected = CarbideExpr::Field(FieldExpr {
                base: Box::new(CarbideExpr::Field(FieldExpr {
                    base: Box::new(CarbideExpr::Path(PathExpr {
                        qself: None,
                        path: parse_quote!(test)
                    })),
                    dot_token: Default::default(),
                    member: CarbideMember::Named(Ident::new("field1", Span::call_site()))
                })),
                dot_token: Default::default(),
                member: CarbideMember::Named(Ident::new("field2", Span::call_site()))
            });

            assert_eq!(expected, actual)
        }

        #[test]
        fn parse_method_call() {
            // Arrange

            let stream = quote!(
                test.method()
            );

            println!("{:#?}", &stream);

            // Act
            let actual: CarbideExpr = syn::parse2(stream).unwrap();

            // Assert
            let expected = CarbideExpr::MethodCall(MethodCallExpr {
                receiver: Box::new(CarbideExpr::Path(PathExpr {
                    qself: None,
                    path: parse_quote!(test)
                })),
                dot_token: Default::default(),
                method: parse_quote!(method),
                turbofish: None,
                paren_token: Default::default(),
                args: Default::default()
            });

            assert_eq!(expected, actual)
        }

        #[test]
        fn parse_state_with_field() {
            // Arrange

            let stream = quote!(
                $test.field1
            );

            println!("{:#?}", &stream);

            // Act
            let actual: CarbideExpr = syn::parse2(stream).unwrap();

            // Assert
            let expected = CarbideExpr::Field(FieldExpr {
                base: Box::new(CarbideExpr::State(StateExpr {
                    dollar_token: Default::default(),
                    expr: Box::new(CarbideExpr::Path(PathExpr {
                        qself: None,
                        path: parse_quote!(test)
                    }))
                })),
                dot_token: Default::default(),
                member: CarbideMember::Named(Ident::new("field1", Span::call_site()))
            });

            assert_eq!(expected, actual)
        }
    }

    mod print_tests {
        
        
        
        

        #[test]
        fn print_ident() {
            // Arrange
            let expr: CarbideExpr = parse_quote!(
                test
            );

            println!("------ Parsed ------");
            println!("{:#?}", &expr);

            // Act
            let token_stream: TokenStream = expr.into_token_stream();
            let actual: Expr = syn::parse2(token_stream).unwrap();

            println!("------ Result ------");
            println!("{}", &actual.to_token_stream().to_string());

            // Assert
            let expected: Expr = parse_quote!(
                test
            );

            assert_eq!(expected, actual)
        }

        #[test]
        fn print_state() {
            // Arrange
            let expr: CarbideExpr = parse_quote!(
                $test
            );

            println!("------ Parsed ------");
            println!("{:#?}", &expr);

            // Act
            let token_stream: TokenStream = expr.into_token_stream();
            let actual: Expr = syn::parse2(token_stream).unwrap();

            println!("------ Result ------");
            println!("{}", &actual.to_token_stream().to_string());

            // Assert
            let expected: Expr = parse_quote!(
                test.clone()
            );

            assert_eq!(expected, actual)
        }

        #[test]
        fn print_field1() {
            // Arrange
            let expr: CarbideExpr = parse_quote!(
                test.field1
            );

            println!("------ Parsed ------");
            println!("{:#?}", &expr);

            // Act
            let token_stream: TokenStream = expr.into_token_stream();
            let actual: Expr = syn::parse2(token_stream).unwrap();

            println!("------ Result ------");
            println!("{}", &actual.to_token_stream().to_string());

            // Assert
            let expected: Expr = parse_quote!(
                carbide_core::state::FieldState::new2(
                    test.clone(),
                    |item| { &item.field1 },
                    |item| { &mut item.field1 }
                )
            );

            assert_eq!(expected, actual)
        }

        #[test]
        fn print_field2() {
            // Arrange
            let expr: CarbideExpr = parse_quote!(
                test.field1.field2
            );

            println!("------ Parsed ------");
            println!("{:#?}", &expr);

            // Act
            let token_stream: TokenStream = expr.into_token_stream();
            let actual: Expr = syn::parse2(token_stream).unwrap();

            println!("------ Result ------");
            println!("{}", &actual.to_token_stream().to_string());

            // Assert
            let expected: Expr = parse_quote!(
                carbide_core::state::FieldState::new2(
                    carbide_core::state::FieldState::new2(
                        test.clone(),
                        |item| { &item.field1 },
                        |item| { &mut item.field1 }
                    ).clone(),
                    |item| { &item.field2 },
                    |item| { &mut item.field2 }
                )
            );

            assert_eq!(expected, actual)
        }

        #[test]
        fn print_index1() {
            // Arrange
            let expr: CarbideExpr = parse_quote!(
                test[test2]
            );

            println!("------ Parsed ------");
            println!("{:#?}", &expr);

            // Act
            let token_stream: TokenStream = expr.into_token_stream();
            let actual: Expr = syn::parse2(token_stream).unwrap();

            println!("------ Result ------");
            println!("{}", &actual.to_token_stream().to_string());

            // Assert
            let expected: Expr = parse_quote!(
                carbide_core::state::IndexState::new2(test.clone(), carbide_core::state::TState::<usize>::from(test2.clone()))
            );

            assert_eq!(expected, actual)
        }

        #[test]
        fn print_index2() {
            // Arrange
            let expr: CarbideExpr = parse_quote!(
                test[0]
            );

            println!("------ Parsed ------");
            println!("{:#?}", &expr);

            // Act
            let token_stream: TokenStream = expr.into_token_stream();
            let actual: Expr = syn::parse2(token_stream).unwrap();

            println!("------ Result ------");
            println!("{}", &actual.to_token_stream().to_string());

            // Assert
            let expected: Expr = parse_quote!(
                carbide_core::state::IndexState::new2(test.clone(), carbide_core::state::TState::<usize>::from(0.clone()))
            );

            assert_eq!(expected, actual)
        }
    }
}
