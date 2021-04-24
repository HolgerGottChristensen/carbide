use syn::token::Brace;

use crate::widget_stmt::WidgetStmt;

pub struct WidgetBlock {
    brace: Brace,
    stmts: Vec<WidgetStmt>,
}