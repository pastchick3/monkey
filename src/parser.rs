use crate::token::Token;
use crate::lexer::Lexer;
use crate::ast::Expression;
use crate::ast::Statement;

// Precedence table.
const LOWEST: u8 = 0;
const EQUALS: u8 = 1;    // ==
const LESSGREATER: u8 = 2;    // < or >
const SUM: u8 = 3;    // +
const PRODUCT: u8 = 4;    // *
const PREFIX: u8 = 5;    // -X or !X
const CALL: u8 = 6;    // function()
const INDEX: u8 = 7;    // arr[0]

pub struct Parser {
    input: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Parser {
        let input = lexer.collect();
        Parser {
            input,
            pos: 0,
        }
    }

    fn token(&self) -> Option<Token> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos].clone())
        } else {
            None
        }
    }

    fn forward(&mut self) -> () {
        self.pos += 1;
    }

    fn assert_and_forward(&mut self, expected: &str) -> String {
        // Assert the current token is of the expected type, then move forward, and
        // finally return this token.
        match self.token() {
            Some(tk) => {
                let s = format!("{:?}", tk);
                let n = match s.find('(') {
                    Some(n) => n,
                    None => panic!("Invalid Token {:?}", tk),
                };
                let name = &s[0..n];    // type
                let value = &s[n+2..s.len()-2];    // value with () and "" stripped
                if expected == name {
                    self.forward();
                    String::from(value)
                } else {
                    panic!(format!("Expect Token::{}, get {:?}.", expected, tk));
                }
            },
            None => panic!(format!("Expect Token::{}, get EOF.", expected)),
        }
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.token() {
            Some(Token::Let(_)) => Some(self.parse_let_statement()),
            Some(Token::Return(_)) => Some(self.parse_return_statement()),
            Some(_) => Some(self.parse_expr_statement()),
            None => None,
        }
    }

    fn parse_let_statement(&mut self) -> Statement {
        self.forward();
        let ident = Expression::Ident(self.assert_and_forward("Ident"));
        self.assert_and_forward("Assign");
        let expr = self.parse_expression(LOWEST);
        self.assert_and_forward("Semicolon");
        Statement::Let { ident, expr }
    }

    fn parse_return_statement(&mut self) -> Statement {
        self.forward();
        let expr = self.parse_expression(LOWEST);
        self.assert_and_forward("Semicolon");
        Statement::Return(expr)
    }

    fn parse_expr_statement(&mut self) -> Statement {
        let expr = self.parse_expression(LOWEST);
        if let Some(Token::Semicolon(_)) = self.token() {
            self.forward();
        }
        Statement::Expr(expr)
    }

    fn parse_expression(&mut self, precedence: u8) -> Expression {
        let mut expr = self.parse_prefix();
        while precedence < self.get_precedence(self.token()) {
            expr = self.parse_infix(expr.clone());
        }
        expr
    }

    fn get_precedence(&self, token: Option<Token>) -> u8 {
        match token {
            Some(Token::Eq(_)) => EQUALS,
            Some(Token::NotEq(_)) => EQUALS,
            Some(Token::LT(_)) => LESSGREATER,
            Some(Token::GT(_)) => LESSGREATER,
            Some(Token::Plus(_)) => SUM,
            Some(Token::Minus(_)) => SUM,
            Some(Token::Slash(_)) => PRODUCT,
            Some(Token::Asterisk(_)) => PRODUCT,
            Some(Token::Lparen(_)) => CALL,
            Some(Token::Lbracket(_)) => INDEX,
            _ => LOWEST,
        }
    }

    fn parse_prefix(&mut self) -> Expression {
        let ch = self.token().unwrap();
        self.forward();
        match ch {
            Token::Ident(ident) => Expression::Ident(ident),
            Token::Int(int) => Expression::Int(int),
            Token::Str(s) => Expression::Str(s),
            Token::True(v) | Token::False(v) => Expression::Bool(v),
            Token::Minus(op) | Token::Bang(op) => Expression::Prefix {
                operator: op,
                expr: Box::new(self.parse_expression(PREFIX)),
            },
            Token::Lparen(_) => {
                let expr = self.parse_expression(LOWEST);
                self.assert_and_forward("Rparen");
                expr
            },
            Token::Lbracket(_) => {
                let mut list = Vec::new();
                match self.token() {
                    Some(Token::Rbracket(_)) => (),
                    _ => loop {
                        list.push(Box::new(self.parse_expression(LOWEST)));
                        match self.token() {
                            Some(Token::Comma(_)) => self.forward(),
                            _ => break,
                        };
                    },
                };
                self.assert_and_forward("Rbracket");
                Expression::Array(list)
            }
            Token::If(_) => {
                self.assert_and_forward("Lparen");
                let condition = self.parse_expression(LOWEST);
                self.assert_and_forward("Rparen");
                self.assert_and_forward("Lbrace");
                let consequence = self.parse_block_statement();
                self.assert_and_forward("Rbrace");
                let alternative = match self.token() {
                    Some(Token::Else(_)) => {
                        self.forward();
                        self.assert_and_forward("Lbrace");
                        let alternative = self.parse_block_statement();
                        self.assert_and_forward("Rbrace");
                        alternative
                    },
                    _ => Statement::Block(Vec::new()),
                };
                Expression::If {
                    condition: Box::new(condition),
                    consequence: Box::new(consequence),
                    alternative: Box::new(alternative),
                }
            },
            Token::Function(_) => {
                self.assert_and_forward("Lparen");
                let mut parameters = Vec::new();
                match self.token() {
                    Some(Token::Rparen(_)) => (),
                    _ => loop {
                        match self.token() {
                            Some(Token::Ident(ident)) => parameters.push(Box::new(Expression::Ident(ident))),
                            tk => panic!(format!("Expect Token::Ident, get {:?}.", tk)),
                        };
                        self.forward();
                        match self.token() {
                            Some(Token::Comma(_)) => self.forward(),
                            _ => break,
                        };
                    },
                };
                self.assert_and_forward("Rparen");
                self.assert_and_forward("Lbrace");
                let body = self.parse_block_statement();
                self.assert_and_forward("Rbrace");
                Expression::Function {
                    parameters,
                    body: Box::new(body),
                }
            },
            tk => panic!(format!("Invalid token: {:?}", tk)),
        }
    }

    fn parse_block_statement(&mut self) -> Statement {
        let mut stmts = Vec::new();
        loop {
            match self.token() {
                Some(Token::Rbrace(_)) => break,
                _ => (),
            };
            stmts.push(match self.parse_statement() {
                Some(stmt) => Box::new(stmt),
                None => panic!("Expect a block statement."),
            });
        };
        Statement::Block(stmts)
    }

    fn parse_infix(&mut self, left: Expression) -> Expression {
        match self.token().unwrap() {
            Token::Lparen(_) => {
                self.forward();
                let mut arguments = Vec::new();
                match self.token() {
                    Some(Token::Rparen(_)) => (),
                    _ => loop {
                        arguments.push(Box::new(self.parse_expression(LOWEST)));
                        match self.token() {
                            Some(Token::Comma(_)) => self.forward(),
                            _ => break,
                        };
                    },
                };
                self.assert_and_forward("Rparen");
                Expression::Call {
                    function: Box::new(left),
                    arguments,
                }
            },
            tk => {
                let precedence = self.get_precedence(Some(tk.clone()));
                let operator = match tk {
                    Token::Eq(op) |
                    Token::NotEq(op) |
                    Token::LT(op) |
                    Token::GT(op) |
                    Token::Plus(op) |
                    Token::Minus(op) |
                    Token::Slash(op) |
                    Token::Asterisk(op) |
                    Token::Lbracket(op) => op,
                    tk => panic!(format!("Invalid token: {:?}", tk)),
                };
                self.forward();
                let right = self.parse_expression(precedence);
                if operator.as_str() == "[" {
                    self.assert_and_forward("Rbracket");
                }
                Expression::Infix {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                }
            },
        }
    }
}

impl Iterator for Parser {

    type Item = Statement;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_statement()
    }
}


#[cfg(test)]
mod tests {

    use super::Lexer;
    use super::Parser;
    use super::Expression;
    use super::Statement;

    #[test]
    fn parser() {
        let input = "
            let x = 10;
            return 1;
            2;
            -3;
            !4;

            5 + 5;
            5 - 5;
            5 * 5;
            5 / 5;
            5 > 5;
            5 < 5;
            5 == 5;
            5 != 5;

            5 + 5 * 5;
            (5 + 5) * 5;

            true;
            !false;

            if (x) {x}
            if (x < y) {
                x
            } else {
                y
            }

            fn() {}
            fn(x, y) {
                x
            }

            add(1, 2 + 3)

            \"a b\";

            [];
            [1];
            [1, 2];
            arr[1];
        ";
        let output = [
            Statement::Let {
                ident: Expression::Ident(String::from("x")),
                expr: Expression::Int(String::from("10")),
            },
            Statement::Return(Expression::Int(String::from("1"))),
            Statement::Expr(Expression::Int(String::from("2"))),
            Statement::Expr(Expression::Prefix {
                operator: String::from("-"),
                expr: Box::new(Expression::Int(String::from("3"))),
            }),
            Statement::Expr(Expression::Prefix {
                operator: String::from("!"),
                expr: Box::new(Expression::Int(String::from("4"))),
            }),

            Statement::Expr(Expression::Infix {
                operator: String::from("+"),
                left: Box::new(Expression::Int(String::from("5"))),
                right: Box::new(Expression::Int(String::from("5"))),
            }),
            Statement::Expr(Expression::Infix {
                operator: String::from("-"),
                left: Box::new(Expression::Int(String::from("5"))),
                right: Box::new(Expression::Int(String::from("5"))),
            }),
            Statement::Expr(Expression::Infix {
                operator: String::from("*"),
                left: Box::new(Expression::Int(String::from("5"))),
                right: Box::new(Expression::Int(String::from("5"))),
            }),
            Statement::Expr(Expression::Infix {
                operator: String::from("/"),
                left: Box::new(Expression::Int(String::from("5"))),
                right: Box::new(Expression::Int(String::from("5"))),
            }),
            Statement::Expr(Expression::Infix {
                operator: String::from(">"),
                left: Box::new(Expression::Int(String::from("5"))),
                right: Box::new(Expression::Int(String::from("5"))),
            }),
            Statement::Expr(Expression::Infix {
                operator: String::from("<"),
                left: Box::new(Expression::Int(String::from("5"))),
                right: Box::new(Expression::Int(String::from("5"))),
            }),
            Statement::Expr(Expression::Infix {
                operator: String::from("=="),
                left: Box::new(Expression::Int(String::from("5"))),
                right: Box::new(Expression::Int(String::from("5"))),
            }),
            Statement::Expr(Expression::Infix {
                operator: String::from("!="),
                left: Box::new(Expression::Int(String::from("5"))),
                right: Box::new(Expression::Int(String::from("5"))),
            }),

            Statement::Expr(Expression::Infix {
                operator: String::from("+"),
                left: Box::new(Expression::Int(String::from("5"))),
                right: Box::new(Expression::Infix {
                    operator: String::from("*"),
                    left: Box::new(Expression::Int(String::from("5"))),
                    right: Box::new(Expression::Int(String::from("5"))),
                }),
            }),
            Statement::Expr(Expression::Infix {
                operator: String::from("*"),
                left: Box::new(Expression::Infix {
                    operator: String::from("+"),
                    left: Box::new(Expression::Int(String::from("5"))),
                    right: Box::new(Expression::Int(String::from("5"))),
                }),
                right: Box::new(Expression::Int(String::from("5"))),
            }),

            Statement::Expr(Expression::Bool(String::from("true"))),
            Statement::Expr(Expression::Prefix {
                operator: String::from("!"),
                expr: Box::new(Expression::Bool(String::from("false"))),
            }),

            Statement::Expr(Expression::If {
                condition: Box::new(Expression::Ident(String::from("x"))),
                consequence: Box::new(Statement::Block(vec!(
                    Box::new(Statement::Expr(Expression::Ident(String::from("x")))),
                ))),
                alternative: Box::new(Statement::Block(Vec::new())),
            }),
            Statement::Expr(Expression::If {
                condition: Box::new(Expression::Infix {
                    operator: String::from("<"),
                    left: Box::new(Expression::Ident(String::from("x"))),
                    right: Box::new(Expression::Ident(String::from("y"))),
                }),
                consequence: Box::new(Statement::Block(vec!(
                    Box::new(Statement::Expr(Expression::Ident(String::from("x")))),
                ))),
                alternative: Box::new(Statement::Block(vec!(
                    Box::new(Statement::Expr(Expression::Ident(String::from("y")))),
                ))),
            }),

            Statement::Expr(Expression::Function {
                parameters: Vec::new(),
                body: Box::new(Statement::Block(Vec::new())),
            }),
            Statement::Expr(Expression::Function {
                parameters: vec!(
                    Box::new(Expression::Ident(String::from("x"))),
                    Box::new(Expression::Ident(String::from("y"))),
                ),
                body: Box::new(Statement::Block(vec!(
                    Box::new(Statement::Expr(Expression::Ident(String::from("x")))),
                ))),
            }),

            Statement::Expr(Expression::Call {
                function: Box::new(Expression::Ident(String::from("add"))),
                arguments: vec!(
                    Box::new(Expression::Int(String::from("1"))),
                    Box::new(Expression::Infix {
                        operator: String::from("+"),
                        left: Box::new(Expression::Int(String::from("2"))),
                        right: Box::new(Expression::Int(String::from("3"))),
                    }),
                ),
            }),

            Statement::Expr(Expression::Str(String::from("a b"))),
            Statement::Expr(Expression::Array(Vec::new())),
            Statement::Expr(Expression::Array(vec!(
                Box::new(Expression::Int(String::from("1"))),
            ))),
            Statement::Expr(Expression::Array(vec!(
                Box::new(Expression::Int(String::from("1"))),
                Box::new(Expression::Int(String::from("2"))),
            ))),
            Statement::Expr(Expression::Infix {
                operator: String::from("["),
                left: Box::new(Expression::Ident(String::from("arr"))),
                right: Box::new(Expression::Int(String::from("1"))),
            }),
        ];
        let lexer = Lexer::new(input);
        let parser = Parser::new(lexer);
        for (result, expected) in parser.zip(output.iter()) {
            println!("Parser: {:?} - {:?}", &result, expected);
            assert_eq!(&result, expected);
        }
    }
}
