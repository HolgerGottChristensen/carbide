use proc_macro2::Ident;

pub enum CarbideExpr {
    Ident(IdentExpr),
    State(StateExpr),
}

pub struct IdentExpr {
    ident: Ident,
}

pub struct StateExpr {
    ident: Ident,
}