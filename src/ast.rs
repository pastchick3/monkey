#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Expression {
    Ident(String),
    Int(String),
    Str(String),
    Bool(String),
    Array(Vec<Box<Expression>>),
    Prefix {
        operator: String,
        expr: Box<Expression>,
    },
    Infix {
        operator: String,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    If {
        condition: Box<Expression>,
        consequence: Box<Statement>,
        alternative: Box<Statement>,
    },
    Function {
        parameters: Vec<Box<Expression>>,
        body: Box<Statement>,
    },
    Call {
        function: Box<Expression>,
        arguments: Vec<Box<Expression>>,
    },
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Statement {
    Let {
        ident: Expression,
        expr: Expression,
    },
    Return(Expression),
    Expr(Expression),
    Block(Vec<Box<Statement>>),
}
