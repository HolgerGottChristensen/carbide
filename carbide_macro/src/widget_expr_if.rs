use syn::token::Else;
use syn::token::If;

use crate::widget_block::WidgetBlock;
use crate::widget_expr::WidgetExpr;

pub struct WidgetExprIf {
    if_token: If,
    cond: Box<WidgetExpr>,
    then_branch: WidgetBlock,
    else_branch: Option<(Else, Box<WidgetExpr>)>,
}