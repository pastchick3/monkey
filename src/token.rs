#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    EOF(String),    // ""

    // identifiers + literals
    Ident(String),    // indentifier
    Int(String),    // integer
    Str(String),    // string

    // operators
    Assign(String),    // "="
    Plus(String),    // "+"
    Minus(String),    // "-"
    Asterisk(String),    // "*"
    Slash(String),    // "/"
    Bang(String),    // "!"
    LT(String),    // "<"
    GT(String),    // ">"
    Eq(String),    // "=="
    NotEq(String),    // "!="

    // delimiters
    Comma(String),    // ","
    Semicolon(String),    // ";"

    Lparen(String),    // "("
    Rparen(String),    // ")"
    Lbrace(String),    // "{"
    Rbrace(String),    // "}"
    Lbracket(String),    // "["
    Rbracket(String),    // "]"

    // keywords
    Function(String),    // "fn"
    Let(String),    // "let"
    If(String),    // "if"
    Else(String),    // "else"
    True(String),    // "true"
    False(String),    // "false"
    Return(String),    // "return"
}
