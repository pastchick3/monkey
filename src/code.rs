use crate::object::Object;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Code {
    Constant(Object),
    Pop,
    Add,
    Sub,
    Mul,
    Div,
    True,
    False,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    Minus,
    Bang,
}
