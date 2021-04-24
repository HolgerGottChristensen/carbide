use syn::{Expr, ExprArray, ExprAssign};

use crate::widget_binding::WidgetBinding;
use crate::widget_expr_for::WidgetExprFor;
use crate::widget_expr_if::WidgetExprIf;
use crate::widget_node::WidgetNode;

pub enum WidgetExpr {
    /// Both the parenthesis and block is optional
    ///
    /// Widget () {
    ///     ...
    /// }
    Node(WidgetNode),

    /// $self.shown
    StateBinding(WidgetBinding),
    For(WidgetExprFor),
    If(WidgetExprIf),
    Expr(Expr),
}