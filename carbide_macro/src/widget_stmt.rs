use syn::{Error, Local};
use syn::parse::{Parse, ParseStream};
use syn::token::Semi;

use crate::widget_expr::WidgetExpr;

pub enum WidgetStmt {
    Local(Local),
    Expr(WidgetExpr),
    Semi(WidgetExpr, Semi),
}

impl Parse for WidgetStmt {
    fn parse(input: ParseStream) -> SynResult<Self> {
        if let Some(local) = input.parse::<Local>().ok() {
            return Ok(WidgetStmt::Local(local));
        }
        if let Some(expr) = input.parse::<WidgetExpr>().ok() {
            return if let Some(semi) = input.parse::<Semi>().ok() {
                Ok(WidgetStmt::Semi(expr, semi))
            } else {
                Ok(WidgetStmt::Expr(expr))
            };

        }
        Err(Error::new(input.span(), "Could not parse WidgetStmt"))
    }
}