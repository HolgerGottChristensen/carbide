use std::mem;
use proc_macro2::{Delimiter, Ident, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::{bracketed, ExprPath, Index, Lit, LitFloat, LitInt, Macro, MacroDelimiter, Path, PathArguments, QSelf, Token};
use syn::token::{Brace, Bracket, Dollar, Paren};

#[derive(PartialEq, Clone, Debug)]
pub enum CarbideExpr {
    Lit(LitExpr),
    Path(PathExpr),
    State(StateExpr),
    Field(FieldExpr),
    Index(IndexExpr),
    Macro(MacroExpr),
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
    dollar_token: Token![$],
    expr: Box<CarbideExpr>,
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

impl ToTokens for CarbideExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            CarbideExpr::Path(i) => {i.to_tokens(tokens)}
            CarbideExpr::State(i) => {i.to_tokens(tokens)}
            CarbideExpr::Field(i) => {i.to_tokens(tokens)}
            CarbideExpr::Index(i) => {i.to_tokens(tokens)}
            CarbideExpr::Lit(i) => {i.to_tokens(tokens)}
            CarbideExpr::Macro(i) => {i.to_tokens(tokens)}
        }
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

        tokens.extend(quote!(
            #lit
        ))
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
            dollar_token,
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
            dot_token,
            member
        } = self;

        tokens.extend(quote!(
            carbide_core::state::FieldState::new2(
                #base.clone(),
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
            bracket_token,
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
        CarbideExpr::parse_expr(input)
    }
}

/// This has been adapted from syn's src/expr.rs
impl CarbideExpr {

    /// <unary>
    fn parse_expr(input: ParseStream) -> syn::Result<CarbideExpr> {
        CarbideExpr::parse_unary_expr(input)
    }

    /// $ <trailer>
    fn parse_unary_expr(input: ParseStream) -> syn::Result<CarbideExpr> {

        if input.peek(Token![$]) {
            let dollar_token: Token![$] = input.parse()?;
            let expr = Box::new(CarbideExpr::parse_unary_expr(input)?);

            Ok(CarbideExpr::State(StateExpr {
                dollar_token,
                expr
            }))
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

                e = CarbideExpr::Field(FieldExpr {
                    base: Box::new(e),
                    dot_token,
                    member: input.parse()?,
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
        {
            CarbideExpr::parse_path_or_macro(input)
        } else {
            Err(input.error("unsupported expression"))
        }
    }

    fn parse_path_or_macro(input: ParseStream) -> syn::Result<CarbideExpr> {
        let expr: PathExpr = input.parse()?;

        if expr.qself.is_none() && input.peek(Token![!]) && !input.peek(Token![!=]) {
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

        Ok(CarbideExpr::Path(expr))
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

    fn print_path(tokens: &mut TokenStream, qself: &Option<QSelf>, path: &Path) {
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
        use proc_macro2::{Ident, Span};
        use quote::quote;
        use syn::parse::Parse;
        use syn::{Lit, parse_quote, Path, PathSegment};
        use syn::punctuated::Punctuated;
        use crate::carbide_expr::{CarbideExpr, CarbideMember, FieldExpr, PathExpr, LitExpr, StateExpr};

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
    }

    mod print_tests {
        use proc_macro2::TokenStream;
        use quote::ToTokens;
        use syn::{Expr, parse_quote};
        use crate::carbide_expr::CarbideExpr;

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
