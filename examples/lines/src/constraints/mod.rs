mod equations;
mod expr;
mod ops;
mod parse;
mod solve;

pub use equations::{Equation, SystemOfEquations};
pub use expr::{BinaryOperation, Expression, Parameter};
pub use parse::{parse, ParseError, TokenKind};
