#[derive(PartialEq, Debug, Clone)]
pub enum Expression {
    Ident(String),
    Int(String),
    Bool(String),
    Prefix {
        operator: String,
        expr: Result<Box<Expression>, String>,
    },
    Infix {
        operator: String,
        left: Result<Box<Expression>, String>,
        right: Result<Box<Expression>, String>,
    },
    If {
        condition: Result<Box<Expression>, String>,
        consequence: Box<Statement>,
        alternative: Box<Statement>,
    },
    Function {
        parameters: Vec<Box<Expression>>,
        body: Box<Statement>,
    },
    Call {
        function: Box<Result<Expression, String>>,
        arguments: Vec<Box<Result<Expression, String>>>,
    },
}

#[derive(PartialEq, Debug, Clone)]
pub enum Statement {
    Let {
        ident: Result<Expression, String>,
        expr: Result<Expression, String>,
    },
    Return(Result<Expression, String>),
    Expr(Result<Expression, String>),
    Block(Vec<Result<Box<Statement>, String>>),
}
