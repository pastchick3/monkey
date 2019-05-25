use crate::token::Token;
use crate::lexer::Lexer;
use crate::ast::Expression;
use crate::ast::Statement;

const LOWEST: u8 = 0;
const EQUALS: u8 = 1;    // ==
const LESSGREATER: u8 = 2;    // < or >
const SUM: u8 = 3;    // +
const PRODUCT: u8 = 4;    // *
const PREFIX: u8 = 5;    // -X or !X
const CALL: u8 = 6;    // function()

pub struct Parser {
    input: Vec<Token>,
    pos: usize,
    next_pos: usize,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Parser {
        let input = lexer.collect();
        Parser {
            input,
            pos: 0,
            next_pos: 1,
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
        self.next_pos += 1;
    }

    fn parse_statement(&mut self) -> Option<Result<Statement, String>> {
        match self.token() {
            Some(Token::Let(_)) => Some(self.parse_let_statement()),
            Some(Token::Return(_)) => Some(self.parse_return_statement()),
            Some(_) => Some(self.parse_expr_statement()),
            None => None,
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement, String> {
        self.forward();
        let ident = match self.token() {
            Some(Token::Ident(ident)) => Ok(Expression::Ident(ident)),
            tk => return Err(format!("Expect Token::Ident, get {:?}.", tk)),
        };
        self.forward();
        match self.token() {
            Some(Token::Assign(_)) => (),
            tk => return Err(format!("Expect Token::Assign, get {:?}.", tk)),
        };
        self.forward();
        let expr = self.parse_expression(LOWEST);
        match self.token() {
            Some(Token::Semicolon(_)) => self.forward(),
            tk => return Err(format!("Expect Token::Semicolon, get {:?}.", tk)),
        };
        Ok(Statement::Let { ident, expr })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, String> {
        self.forward();
        let expr = self.parse_expression(LOWEST);
        match self.token() {
            Some(Token::Semicolon(_)) => self.forward(),
            tk => return Err(format!("Expect Token::Semicolon, get {:?}.", tk)),
        };
        Ok(Statement::Return(expr))
    }

    fn parse_expr_statement(&mut self) -> Result<Statement, String> {
        let expr = self.parse_expression(LOWEST);
        if let Some(Token::Semicolon(_)) = self.token() {
            self.forward();
        }
        Ok(Statement::Expr(expr))
    }

    fn parse_expression(&mut self, precedence: u8) -> Result<Expression, String> {
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
            _ => LOWEST,
        }
    }

    fn parse_prefix(&mut self) -> Result<Expression, String> {
        match self.token().unwrap() {
            Token::Ident(ident) => {
                self.forward();
                Ok(Expression::Ident(ident))
            },
            Token::Int(int) => {
                self.forward();
                Ok(Expression::Int(int))
            },
            Token::True(v) | Token::False(v) => {
                self.forward();
                Ok(Expression::Bool(v))
            },
            Token::Minus(operator) | Token::Bang(operator) => {
                self.forward();
                let expr = match self.parse_expression(PREFIX) {
                    Ok(expr) => Ok(Box::new(expr)),
                    Err(s) => Err(s),
                };
                Ok(Expression::Prefix {
                    operator,
                    expr,
                })
            },
            Token::Lparen(_) => {
                self.forward();
                let expr = self.parse_expression(LOWEST);
                match self.token() {
                    Some(Token::Rparen(_)) => self.forward(),
                    tk => return Err(format!("Expect Token::Rparen, get {:?}.", tk)),
                };
                expr
            },
            Token::If(_) => {
                self.forward();
                match self.token() {
                    Some(Token::Lparen(_)) => self.forward(),
                    tk => return Err(format!("Expect Token::Lparen, get {:?}.", tk)),
                };
                let condition = match self.parse_expression(LOWEST) {
                    Ok(expr) => Ok(Box::new(expr)),
                    Err(s) => Err(s),
                };
                match self.token() {
                    Some(Token::Rparen(_)) => self.forward(),
                    tk => return Err(format!("Expect Token::Rparen, get {:?}.", tk)),
                };
                match self.token() {
                    Some(Token::Lbrace(_)) => self.forward(),
                    tk => return Err(format!("Expect Token::Lbrace, get {:?}.", tk)),
                };
                let consequence = self.parse_block_statement();
                match self.token() {
                    Some(Token::Rbrace(_)) => self.forward(),
                    tk => return Err(format!("Expect Token::Rbrace, get {:?}.", tk)),
                };
                let alternative = match self.token() {
                    Some(Token::Else(_)) => {
                        self.forward();
                        match self.token() {
                            Some(Token::Lbrace(_)) => self.forward(),
                            tk => return Err(format!("Expect Token::Lbrace, get {:?}.", tk)),
                        };
                        let alternative = self.parse_block_statement();
                        match self.token() {
                            Some(Token::Rbrace(_)) => self.forward(),
                            tk => return Err(format!("Expect Token::Rbrace, get {:?}.", tk)),
                        };
                        alternative
                    },
                    _ => Box::new(Statement::Block(Vec::new())),
                };
                Ok(Expression::If {
                    condition,
                    consequence,
                    alternative,
                })
            },
            Token::Function(_) => {
                self.forward();
                match self.token() {
                    Some(Token::Lparen(_)) => self.forward(),
                    tk => return Err(format!("Expect Token::Lparen, get {:?}.", tk)),
                };
                let mut parameters = Vec::new();
                match self.token() {
                    Some(Token::Rparen(_)) => (),
                    _ => loop {
                        match self.token() {
                            Some(Token::Ident(ident)) => parameters.push(Box::new(Expression::Ident(ident))),
                            _ => (),
                        };
                        self.forward();
                        match self.token() {
                            Some(Token::Comma(_)) => self.forward(),
                            _ => break,
                        };
                    },
                };
                match self.token() {
                    Some(Token::Rparen(_)) => self.forward(),
                    tk => return Err(format!("Expect Token::Rparen, get {:?}.", tk)),
                };
                match self.token() {
                    Some(Token::Lbrace(_)) => self.forward(),
                    tk => return Err(format!("Expect Token::Lbrace, get {:?}.", tk)),
                };
                let body = self.parse_block_statement();
                match self.token() {
                    Some(Token::Rbrace(_)) => self.forward(),
                    tk => return Err(format!("Expect Token::Rbrace, get {:?}.", tk)),
                };
                Ok(Expression::Function {
                    parameters,
                    body,
                })
            },
            token => Err(format!("Invalid token: {:?}", token)),
        }
    }

    fn parse_block_statement(&mut self) -> Box<Statement> {
        let mut stmts = Vec::new();
        loop {
            match self.token() {
                Some(Token::Rbrace(_)) | None => break,
                _ => (),
            }
            stmts.push(match self.parse_statement() {
                Some(Ok(stmt)) => Ok(Box::new(stmt)),
                Some(Err(s)) => Err(s),
                None => Err(String::from("Expect a block statement.")),
            });
        };
        Box::new(Statement::Block(stmts))
    }

    fn parse_infix(&mut self, left: Result<Expression, String>) -> Result<Expression, String> {
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
                }
                Ok(Expression::Call {
                    function: Box::new(left),
                    arguments,
                })
            },
            _ => {
                let precedence = self.get_precedence(self.token());
                let operator = match self.token().unwrap() {
                    Token::Eq(operator) |
                    Token::NotEq(operator) |
                    Token::LT(operator) |
                    Token::GT(operator) |
                    Token::Plus(operator) |
                    Token::Minus(operator) |
                    Token::Slash(operator) |
                    Token::Asterisk(operator) => operator,
                    token => return Err(format!("Invalid token: {:?}", token)),
                };
                let left = match left {
                    Ok(expr) => Ok(Box::new(expr)),
                    Err(s) => Err(s),
                };
                self.forward();
                let right = match self.parse_expression(precedence) {
                    Ok(expr) => Ok(Box::new(expr)),
                    Err(s) => Err(s),
                };
                Ok(Expression::Infix {
                    operator,
                    left,
                    right,
                })
            },
        }
    }
}

impl Iterator for Parser {

    type Item = Result<Statement, String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_statement()
    }
}


#[cfg(test)]
mod tests {

    use super::Lexer;
    use super::Expression;
    use super::Statement;
    use super::Parser;

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
        ";
        let output = [
            Statement::Let {
                ident: Ok(Expression::Ident(String::from("x"))),
                expr: Ok(Expression::Int(String::from("10"))),
            },
            Statement::Return(Ok(Expression::Int(String::from("1")))),
            Statement::Expr(Ok(Expression::Int(String::from("2")))),
            Statement::Expr(Ok(Expression::Prefix {
                operator: String::from("-"),
                expr: Ok(Box::new(Expression::Int(String::from("3")))),
            })),
            Statement::Expr(Ok(Expression::Prefix {
                operator: String::from("!"),
                expr: Ok(Box::new(Expression::Int(String::from("4")))),
            })),

            Statement::Expr(Ok(Expression::Infix {
                operator: String::from("+"),
                left: Ok(Box::new(Expression::Int(String::from("5")))),
                right: Ok(Box::new(Expression::Int(String::from("5")))),
            })),
            Statement::Expr(Ok(Expression::Infix {
                operator: String::from("-"),
                left: Ok(Box::new(Expression::Int(String::from("5")))),
                right: Ok(Box::new(Expression::Int(String::from("5")))),
            })),
            Statement::Expr(Ok(Expression::Infix {
                operator: String::from("*"),
                left: Ok(Box::new(Expression::Int(String::from("5")))),
                right: Ok(Box::new(Expression::Int(String::from("5")))),
            })),
            Statement::Expr(Ok(Expression::Infix {
                operator: String::from("/"),
                left: Ok(Box::new(Expression::Int(String::from("5")))),
                right: Ok(Box::new(Expression::Int(String::from("5")))),
            })),
            Statement::Expr(Ok(Expression::Infix {
                operator: String::from(">"),
                left: Ok(Box::new(Expression::Int(String::from("5")))),
                right: Ok(Box::new(Expression::Int(String::from("5")))),
            })),
            Statement::Expr(Ok(Expression::Infix {
                operator: String::from("<"),
                left: Ok(Box::new(Expression::Int(String::from("5")))),
                right: Ok(Box::new(Expression::Int(String::from("5")))),
            })),
            Statement::Expr(Ok(Expression::Infix {
                operator: String::from("=="),
                left: Ok(Box::new(Expression::Int(String::from("5")))),
                right: Ok(Box::new(Expression::Int(String::from("5")))),
            })),
            Statement::Expr(Ok(Expression::Infix {
                operator: String::from("!="),
                left: Ok(Box::new(Expression::Int(String::from("5")))),
                right: Ok(Box::new(Expression::Int(String::from("5")))),
            })),

            Statement::Expr(Ok(Expression::Infix {
                operator: String::from("+"),
                left: Ok(Box::new(Expression::Int(String::from("5")))),
                right: Ok(Box::new(Expression::Infix {
                    operator: String::from("*"),
                    left: Ok(Box::new(Expression::Int(String::from("5")))),
                    right: Ok(Box::new(Expression::Int(String::from("5")))),
                })),
            })),
            Statement::Expr(Ok(Expression::Infix {
                operator: String::from("*"),
                left: Ok(Box::new(Expression::Infix {
                    operator: String::from("+"),
                    left: Ok(Box::new(Expression::Int(String::from("5")))),
                    right: Ok(Box::new(Expression::Int(String::from("5")))),
                })),
                right: Ok(Box::new(Expression::Int(String::from("5")))),
            })),

            Statement::Expr(Ok(Expression::Bool(String::from("true")))),
            Statement::Expr(Ok(Expression::Prefix {
                operator: String::from("!"),
                expr: Ok(Box::new(Expression::Bool(String::from("false")))),
            })),

            Statement::Expr(Ok(Expression::If {
                condition: Ok(Box::new(Expression::Ident(String::from("x")))),
                consequence: Box::new(Statement::Block(vec!(
                    Ok(Box::new(Statement::Expr(Ok(Expression::Ident(String::from("x")))))),
                ))),
                alternative: Box::new(Statement::Block(Vec::new())),
            })),
            Statement::Expr(Ok(Expression::If {
                condition: Ok(Box::new(Expression::Infix {
                    operator: String::from("<"),
                    left: Ok(Box::new(Expression::Ident(String::from("x")))),
                    right: Ok(Box::new(Expression::Ident(String::from("y")))),
                })),
                consequence: Box::new(Statement::Block(vec!(
                    Ok(Box::new(Statement::Expr(Ok(Expression::Ident(String::from("x")))))),
                ))),
                alternative: Box::new(Statement::Block(vec!(
                    Ok(Box::new(Statement::Expr(Ok(Expression::Ident(String::from("y")))))),
                ))),
            })),

            Statement::Expr(Ok(Expression::Function {
                parameters: Vec::new(),
                body: Box::new(Statement::Block(Vec::new())),
            })),
            Statement::Expr(Ok(Expression::Function {
                parameters: vec!(
                    Box::new(Expression::Ident(String::from("x"))),
                    Box::new(Expression::Ident(String::from("y"))),
                ),
                body: Box::new(Statement::Block(vec!(
                    Ok(Box::new(Statement::Expr(Ok(Expression::Ident(String::from("x")))))),
                ))),
            })),

            Statement::Expr(Ok(Expression::Call {
                function: Box::new(Ok(Expression::Ident(String::from("add")))),
                arguments: vec!(
                    Box::new(Ok(Expression::Int(String::from("1")))),
                    Box::new(Ok(Expression::Infix {
                        operator: String::from("+"),
                        left: Ok(Box::new(Expression::Int(String::from("2")))),
                        right: Ok(Box::new(Expression::Int(String::from("3")))),
                    })),
                ),
            })),
        ];
        let lexer = Lexer::new(input);
        let parser = Parser::new(lexer);
        for (result, expected) in parser.zip(output.iter()) {
            println!("Parser: {:?} - {:?}", &result, expected);
            assert_eq!(&result.unwrap(), expected);
        }
    }
}
