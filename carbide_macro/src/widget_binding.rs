use syn::Expr;
use syn::token::Dollar;

pub struct WidgetBinding {
    dollar: Dollar,
    expr: Box<Expr>,
}