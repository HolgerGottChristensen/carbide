use std::fmt::{Debug, Formatter, Pointer};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Arm, Attribute, Block, Error, ExprForLoop, ExprIf, Ident, Pat, PatOr};
use syn::token::{Brace, Colon, Comma, Dot, Else, In, Let, Paren, Token};
use syn::{braced, Expr, parenthesized, Token, Type};
use syn::__private::{parse_braces, parse_parens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use crate::carbide_expression::CarbideExpression::{ForLoop, If, Instantiate, Match};
use crate::carbide_expression::CarbideInstantiateParam::{Optional, Required};
use crate::ident_extraction::extract_idents_from_pattern;

#[derive(Debug)]
pub enum CarbideExpression {
    Instantiate(CarbideInstantiate),
    If(CarbideExprIf),
    ForLoop(CarbideExprForLoop),
    Match(CarbideExprMatch)
}

pub struct CarbideExprMatch {
    pub attrs: Vec<Attribute>,
    pub match_token: Token![match],
    pub expr: Box<Expr>,
    pub brace_token: Brace,
    pub arms: Vec<CarbideArm>,
}

pub struct CarbideArm {
    pub attrs: Vec<Attribute>,
    pub pat: Pat,
    pub guard: Option<(Token![if], Box<Expr>)>,
    pub fat_arrow_token: Token![=>],
    pub body: Box<CarbideBlock>,
    pub comma: Option<Token![,]>,
}

impl ToTokens for CarbideExprMatch {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let CarbideExprMatch {
            attrs,
            match_token,
            expr,
            brace_token,
            arms
        } = self;

        let arms = arms.iter().map(|arm| {
            CarbideArm::tokens(arm, *expr.clone())
        });

        tokens.extend(quote!(
            {
                let into_state = TState::from({#expr});
                Match::new(into_state.clone())
                #(#arms)*
            }

        ))
    }
}

impl CarbideArm {
    fn tokens(&self, match_expr: Expr) -> TokenStream {
        let CarbideArm {
            attrs,
            pat,
            guard,
            fat_arrow_token,
            body,
            comma
        } = self;

        let idents = extract_idents_from_pattern(pat.clone());

        let quoted_idents = if idents.len() == 0 {
            quote!()
        } else {
            quote!(
                #(#idents),* =>
            )
        };

        let body = if body.exprs.len() == 0 {
            quote!(#body)
        } else if body.exprs.len() == 1 {
            let item = &body.exprs[0];
            quote!(#item)
        } else {
            quote!(ZStack::new(#body))
        };

        let guard = if let Some((_, expr)) = guard {
            quote!(if #expr)
        } else {
            quote!()
        };

        quote!(
            .case({let into_state_cl = into_state.clone(); carbide_core::matches_case!(into_state_cl, #pat #guard, #quoted_idents {#body})})
        )
    }
}

impl Debug for CarbideExprMatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CarbideExprMatch")
            .field("expr", &self.expr.to_token_stream().to_string())
            .field("arms", &self.arms)
            .finish()
    }
}

impl Debug for CarbideArm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CarbideArm")
            .field("pat", &self.pat.to_token_stream().to_string())
            .field("guard", &self.guard.as_ref().map(|a| a.1.to_token_stream().to_string()).unwrap_or("".to_string()))
            .field("body", &self.body)
            .finish()
    }
}

impl Parse for CarbideExprMatch {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attrs = input.call(Attribute::parse_outer)?;
        let match_token: Token![match] = input.parse()?;
        let expr = Expr::parse_without_eager_brace(input)?;

        let content;
        let brace_token = braced!(content in input);
        //attr::parsing::parse_inner(&content, &mut attrs)?;

        let mut arms = Vec::new();
        while !content.is_empty() {
            arms.push(content.call(CarbideArm::parse)?);
        }

        Ok(CarbideExprMatch {
            attrs,
            match_token,
            expr: Box::new(expr),
            brace_token,
            arms,
        })
    }
}

impl Parse for CarbideArm {
    fn parse(input: ParseStream) -> syn::Result<CarbideArm> {
        let requires_comma;
        Ok(CarbideArm {
            attrs: input.call(Attribute::parse_outer)?,
            pat: CarbideExprForLoop::multi_pat_with_leading_vert(input)?,
            guard: {
                if input.peek(Token![if]) {
                    let if_token: Token![if] = input.parse()?;
                    let guard: Expr = input.parse()?;
                    Some((if_token, Box::new(guard)))
                } else {
                    None
                }
            },
            fat_arrow_token: input.parse()?,
            body: {
                let body = CarbideBlock::parse(input)?;
                requires_comma = false;
                Box::new(body)
            },
            comma: {
                if requires_comma && !input.is_empty() {
                    Some(input.parse()?)
                } else {
                    input.parse()?
                }
            },
        })
    }
}

pub struct CarbideExprForLoop {
    pub attrs: Vec<Attribute>,
    pub for_token: Token![for],
    pub pat: Pat,
    pub in_token: Token![in],
    pub expr: Box<Expr>,
    pub body: CarbideBlock,
}

impl ToTokens for CarbideExprForLoop {
    fn to_tokens(&self, tokens: &mut TokenStream) {

        let CarbideExprForLoop {
            attrs,
            for_token,
            pat,
            in_token,
            expr,
            body
        } = self;

        tokens.extend(quote!(
            ForEach::new(
                #expr,
                |item: TState<_>, _| -> Box<dyn Widget> {
                    //let #pat = &*item.value();
                    let #pat = item;
                    ZStack::new(
                        #body
                    )
                }
            )
        ))
    }
}

impl Debug for CarbideExprForLoop {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CarbideExprForLoop")
            .field("pat", &self.pat.to_token_stream().to_string())
            .field("expr", &self.expr.to_token_stream().to_string())
            .field("body", &self.body)
            .finish()
    }
}

impl Parse for CarbideExprForLoop {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attrs = input.call(Attribute::parse_outer)?;

        let for_token: Token![for] = input.parse()?;

        let pat = CarbideExprForLoop::multi_pat_with_leading_vert(input)?;

        let in_token: Token![in] = input.parse()?;
        let expr: Expr = input.call(Expr::parse_without_eager_brace)?;

        let body = CarbideBlock::parse(input)?;

        Ok(CarbideExprForLoop {
            attrs,
            for_token,
            pat,
            in_token,
            expr: Box::new(expr),
            body,
        })
    }
}

/// The code in this impl is copied from syn::pat::parsing, since they dont expose it
/// and the goal is to parse stuff as close to syn as possible
impl CarbideExprForLoop {
    fn multi_pat_with_leading_vert(input: ParseStream) -> syn::Result<Pat> {
        let leading_vert: Option<Token![|]> = input.parse()?;
        CarbideExprForLoop::multi_pat_impl(input, leading_vert)
    }

    fn multi_pat_impl(input: ParseStream, leading_vert: Option<Token![|]>) -> syn::Result<Pat> {
        let mut pat: Pat = input.parse()?;
        if leading_vert.is_some()
            || input.peek(Token![|]) && !input.peek(Token![||]) && !input.peek(Token![|=])
        {
            let mut cases = Punctuated::new();
            cases.push_value(pat);
            while input.peek(Token![|]) && !input.peek(Token![||]) && !input.peek(Token![|=]) {
                let punct = input.parse()?;
                cases.push_punct(punct);
                let pat: Pat = input.parse()?;
                cases.push_value(pat);
            }
            pat = Pat::Or(PatOr {
                attrs: Vec::new(),
                leading_vert,
                cases,
            });
        }
        Ok(pat)
    }
}

pub struct CarbideExprIf {
    pub attrs: Vec<Attribute>,
    pub if_token: Token![if],
    pub cond: Box<Expr>,
    pub then_branch: CarbideBlock,
    pub else_branch: CarbideElseBranch,
}

pub enum CarbideElseBranch {
    ElseIf (Token![else], Box<CarbideExprIf>),
    Else (Token![else], CarbideBlock),
    None
}

pub struct CarbideBlock {
    pub brace_token: Brace,
    pub exprs: Vec<CarbideExpression>,
}

impl Debug for CarbideExprIf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CarbideExprIf")
            .field("expr", &self.cond.to_token_stream().to_string())
            .field("then", &self.then_branch)
            .field("else", &self.else_branch)
            .finish()
    }
}

impl Debug for CarbideElseBranch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CarbideElseBranch::ElseIf(_, if_expr) => {
                Debug::fmt(if_expr, f)
            }
            CarbideElseBranch::Else(_, el) => {
                Debug::fmt(el, f)
            }
            CarbideElseBranch::None => {
                f.write_str("None")
            }
        }
    }
}

impl Debug for CarbideBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CarbideBlock")
            .field("exprs", &self.exprs)
            .finish()
    }
}

impl Parse for CarbideExprIf {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        Ok(CarbideExprIf {
            attrs,
            if_token: input.parse()?,
            cond: Box::new(input.call(Expr::parse_without_eager_brace)?),
            then_branch: input.parse()?,
            else_branch: {
                if !input.peek(Token![else]) {
                    CarbideElseBranch::None
                } else if input.peek2(Token![if]) {
                    println!("Else if");
                    CarbideElseBranch::ElseIf(Else::parse(input)?, Box::new(CarbideExprIf::parse(input)?))
                } else {
                    println!("Else");
                    CarbideElseBranch::Else(Else::parse(input)?, CarbideBlock::parse(input)?)
                }
            },
        })
    }
}


impl Parse for CarbideBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let brace_content;
        if let Ok(brace) = parse_braces(input) {
            brace_content = brace.content;

            let mut body = vec![];

            while let Ok(expr) = CarbideExpression::parse(&brace_content) {
                body.push(expr)
            }

            Ok(CarbideBlock {
                brace_token: brace.token,
                exprs: body
            })
        } else {
            panic!("The block should have braces. Otherwise its not a block")
        }
    }
}

impl ToTokens for CarbideBlock {
    fn to_tokens(&self, tokens: &mut TokenStream) {

        let CarbideBlock {
            brace_token,
            exprs
        } = self;

        if exprs.len() == 0 {
            tokens.extend(quote!(
                Empty::new()
            ))
        } else {
            tokens.extend(quote!(vec![
                #(#exprs,)*
            ]))
        }
    }
}

impl ToTokens for CarbideExprIf {
    fn to_tokens(&self, tokens: &mut TokenStream) {

        let CarbideExprIf {
            attrs,
            if_token,
            cond,
            then_branch,
            else_branch
        } = self;

        let else_quote = match else_branch {
            CarbideElseBranch::ElseIf(_, e) => {
                quote!(.when_false(#e))
            }
            CarbideElseBranch::Else(_, e) => {
                quote!(.when_false(ZStack::new(#e)))
            }
            CarbideElseBranch::None => {
                quote!()
            }
        };

        tokens.extend(quote!(
            IfElse::new(#cond)
                .when_true(ZStack::new(#then_branch))
                #else_quote
        ))
    }
}

impl ToTokens for CarbideExpression {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Instantiate(i) => {
                tokens.extend(quote!(
                    #i
                ))
            }
            If(i) => {
                tokens.extend(quote!(
                    #i
                ))
            }
            ForLoop(i) => {
                tokens.extend(quote!(
                    #i
                ))
            }
            Match(i) => {
                tokens.extend(quote!(
                    #i
                ))
            }
        }
    }
}

// Todo: we should probably look more into how to choose the different expressions and not just looking at the first token
impl Parse for CarbideExpression {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token!(match)) {
            let match_expr = CarbideExprMatch::parse(input)?;
            Ok(Match(match_expr))
        } else if input.peek(Token!(for)) {
            let for_expr = CarbideExprForLoop::parse(input)?;
            Ok(ForLoop(for_expr))
        } else if input.peek(Token!(if)) {
            let if_expr = CarbideExprIf::parse(input)?;
            Ok(If(if_expr))
        } else {
            let instantiate = CarbideInstantiate::parse(input)?;
            Ok(Instantiate(instantiate))
        }
    }
}

pub struct CarbideInstantiate {
    ident: Ident,
    paren: Option<Paren>,
    params: Option<Punctuated<CarbideInstantiateParam, Token![,]>>,
    braces: Option<Brace>,
    iterate: Option<CarbideInstantiateIterate>,
    body: Option<Vec<CarbideExpression>>,
    modifiers: Vec<CarbideInstantiateModifier>,
}

impl ToTokens for CarbideInstantiate {
    fn to_tokens(&self, tokens: &mut TokenStream) {

        let CarbideInstantiate {
            ident,
            body,
            params,
            modifiers,
            ..
        } = self;

        let children = if let Some(body) = body {
            quote!(vec![
                #(#body,)*
            ],)
        } else {
            quote!()
        };

        let required = if let Some(params) = params {

            let mapped = params.iter().filter_map(|param| {
                param.required_init_field()
            });

            quote!(
                #(#mapped,)*
            )
        } else {
            quote!()
        };

        let optional = if let Some(params) = params {
            let mapped = params.iter().filter_map(|param| {
                param.optional_init_function()
            });

            quote!(
                #(#mapped)*
            )
        } else {
            quote!()
        };

        tokens.extend(quote!(
            #ident::builder(
                #children
                #required
            )#optional
            .finish()
            #(#modifiers)*
        ))
    }
}

impl Debug for CarbideInstantiate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CarbideInstantiate")
            .field("ident", &self.ident.to_string())
            .field("params", &self.params)
            .field("iterate", &self.iterate)
            .field("body", &self.body)
            .field("modifiers", &self.modifiers)
            .finish()
    }
}

impl Parse for CarbideInstantiate {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = Ident::parse(input)?;


        let paren_content;

        let (paren, params) = if let Ok(paren) = parse_parens(input) {
            paren_content = paren.content;

            let params = {
                use syn::parse_quote::ParseQuote;
                Punctuated::<CarbideInstantiateParam, Token![,]>::parse(&paren_content)?
            };

            (Some(paren.token), Some(params))
        } else {
            (None, None)
        };

        let brace_content;

        let (braces, body, iterate) = if let Ok(brace) = parse_braces(input) {
            brace_content = brace.content;

            let iterate = CarbideInstantiateIterate::parse(&brace_content).ok();

            let mut body = vec![];

            while let Ok(expr) = CarbideExpression::parse(&brace_content) {
                body.push(expr)
            }

            (Some(brace.token), Some(body), iterate)
        } else {
            (None, None, None)
        };

        let mut modifiers = vec![];

        while let Ok(expr) = CarbideInstantiateModifier::parse(input) {
            modifiers.push(expr)
        }

        Ok(CarbideInstantiate {
            ident,
            paren,
            params,
            braces,
            body,
            iterate,
            modifiers
        })
    }
}

pub struct CarbideInstantiateModifier {
    dot_token: Token![.],
    ident: Ident,
    paren_token: Paren,
    exprs: Punctuated<Expr, Comma>,
}

impl ToTokens for CarbideInstantiateModifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {

        let CarbideInstantiateModifier {
            ident,
            exprs,
            ..
        } = self;

        tokens.extend(quote!(
            .#ident(#exprs)
        ))
    }
}

impl Debug for CarbideInstantiateModifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CarbideInstantiateModifier")
            .field("ident", &self.ident.to_string())
            .field("exprs", &self.exprs.to_token_stream().to_string())
            .finish()
    }
}

impl Parse for CarbideInstantiateModifier {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token!(.)) && input.peek2(Ident) && input.peek3(Paren) {
            let dot_token = Dot::parse(input)?;
            let ident = Ident::parse(input)?;

            let paren_content;
            let paren_token = parenthesized!(paren_content in input);

            let exprs = {
                use syn::parse_quote::ParseQuote;
                Punctuated::<Expr, Token![,]>::parse(&paren_content)?
            };

            Ok(CarbideInstantiateModifier {
                dot_token,
                ident,
                paren_token,
                exprs
            })
        } else {
            Err(Error::new(Span::call_site(), "Could not parse carbide modifier"))
        }
    }
}

pub struct CarbideInstantiateIterate {
    vars: Vec<Ident>,
    in_token: Token![in],
}

impl Debug for CarbideInstantiateIterate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CarbideInstantiateIterate")
            .field("ident", &self.vars[0].to_string())
            .finish()
    }
}

impl Parse for CarbideInstantiateIterate {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        let (ident, in_token)= if input.peek(Ident) && input.peek2(Token!(in)) {
            let ident = Ident::parse(input)?;
            let in_token = In::parse(input)?;
            (ident, in_token)
        } else {
            Err(Error::new(Span::call_site(), "Could not parse carbide iterate"))?
        };

        Ok(CarbideInstantiateIterate {
            vars: vec![ident],
            in_token
        })
    }
}

pub enum CarbideInstantiateParam {
    Required {
        expr: Expr,
    },
    Optional {
        ident: Ident,
        colon: Colon,
        expr: Expr,
    }
}

impl CarbideInstantiateParam {
    fn required_init_field(&self) -> Option<TokenStream> {
        match self {
            CarbideInstantiateParam::Required { expr } => {
                Some(quote!({
                    #expr
                }))
            }
            CarbideInstantiateParam::Optional { .. } => None,
        }
    }

    fn optional_init_function(&self) -> Option<TokenStream> {
        match self {
            CarbideInstantiateParam::Optional { ident, expr, .. } => {

                let ident = Ident::new(&format!("with_optional_{}", ident.to_string()), ident.span());

                Some(quote!(
                    .#ident({#expr})
                ))
            }
            CarbideInstantiateParam::Required { .. } => None,
        }
    }
}

impl Debug for CarbideInstantiateParam {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CarbideInstantiateParam::Required { expr } => {
                f.debug_struct("Required")
                    .field("expr", &expr.to_token_stream().to_string())
                    .finish()
            }
            CarbideInstantiateParam::Optional { ident, expr, .. } => {
                f.debug_struct("Optional")
                    .field("ident", &ident.to_string())
                    .field("expr", &expr.to_token_stream().to_string())
                    .finish()
            }
        }
    }
}

impl Parse for CarbideInstantiateParam {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let res = if input.peek(Ident) && input.peek2(Token!(:)) {
            Optional {
                ident: Ident::parse(input)?,
                colon: Colon::parse(input)?,
                expr: Expr::parse(input)?
            }
        } else {
            Required {
                expr: Expr::parse(input)?
            }
        };

        Ok(res)
    }
}