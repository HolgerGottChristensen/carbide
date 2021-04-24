use syn::Pat;
use syn::token::For;
use syn::token::In;

use crate::widget_block::WidgetBlock;
use crate::widget_expr::WidgetExpr;

pub struct WidgetExprFor {
    for_token: For,
    pat: Pat,
    // Might change to WidgetPat
    in_token: In,
    expr: Box<WidgetExpr>,
    body: WidgetBlock,
}