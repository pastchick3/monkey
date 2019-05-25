use crate::token::Token;

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    next_pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Lexer{
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            next_pos: 1,
        }
    }

    fn ch(&mut self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn next_ch(&mut self) -> Option<char> {
        if self.next_pos < self.input.len() {
            Some(self.input[self.next_pos])
        } else {
            None
        }
    }

    fn forward(&mut self) -> () {
        self.pos += 1;
        self.next_pos += 1;
    }

    fn backward(&mut self) -> () {
        self.pos -= 1;
        self.next_pos -= 1;
    }

    fn read_word(&mut self, ch: char) -> Token {
        let mut s = String::new();
        if ch.is_ascii_digit() {
            loop {
                match self.ch() {
                    Some(ch) => {
                        if ch.is_ascii_digit() {
                            s.push(ch);
                        } else {
                            self.backward();
                            return Token::Int(s);
                        }
                    },
                    None => {
                        self.backward();
                        return Token::Int(s);
                    },
                }
                self.forward();
            }
        } else {
            loop {
                match self.ch() {
                    Some(ch) => {
                        if ch.is_ascii_alphanumeric() || ch == '_' {
                            s.push(ch);
                        } else {
                            break;
                        }
                    },
                    None => break,
                }
                self.forward();
            }
            self.backward();
            match s.as_str() {
                "fn" => Token::Function(s),
                "let" => Token::Let(s),
                "if" => Token::If(s),
                "else" => Token::Else(s),
                "true" => Token::True(s),
                "false" => Token::False(s),
                "return" => Token::Return(s),
                _ => Token::Ident(s),
            }
        }
    }
}

impl Iterator for Lexer {

    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let ch = self.ch();
            if ch.is_some() && ch.unwrap().is_whitespace() {
                self.forward();
            } else {
                break;
            }
        }
        let token = match self.ch() {
            Some('=') => {
                match self.next_ch() {
                    Some('=') => {
                        self.forward();
                        Some(Token::Eq(String::from("==")))
                    },
                    _ => Some(Token::Assign(String::from("="))),
                }
            },
            Some('!') => {
                match self.next_ch() {
                    Some('=') => {
                        self.forward();
                        Some(Token::NotEq(String::from("!=")))
                    },
                    _ => Some(Token::Bang(String::from("!"))),
                }
            },
            Some('+') => Some(Token::Plus(String::from("+"))),
            Some('-') => Some(Token::Minus(String::from("-"))),
            Some('*') => Some(Token::Asterisk(String::from("*"))),
            Some('/') => Some(Token::Slash(String::from("/"))),
            Some('<') => Some(Token::LT(String::from("<"))),
            Some('>') => Some(Token::GT(String::from(">"))),
            Some('(') => Some(Token::Lparen(String::from("("))),
            Some(')') => Some(Token::Rparen(String::from(")"))),
            Some('{') => Some(Token::Lbrace(String::from("{"))),
            Some('}') => Some(Token::Rbrace(String::from("}"))),
            Some(',') => Some(Token::Comma(String::from(","))),
            Some(';') => Some(Token::Semicolon(String::from(";"))),
            Some('\0') => Some(Token::EOF(String::from(""))),
            None => None,
            Some(ch) => Some(self.read_word(ch)),
        };
        self.forward();
        token
    }
}


#[cfg(test)]
mod tests {

    use super::Token;
    use super::Lexer;
    
    #[test]
    fn lexer() {
        let input = "
            let five = 5;

            let add = fn(x, y) {
                x + y;
            };

            !-/*5;
            5 < 10 > 5;

            if (5 < 10) {
                return true;
            } else {
                return false;
            }

            10 == 10;
            10 != 9;
        ";
        let output = [
            Token::Let(String::from("let")),
            Token::Ident(String::from("five")),
            Token::Assign(String::from("=")),
            Token::Int(String::from("5")),
            Token::Semicolon(String::from(";")),

            Token::Let(String::from("let")),
            Token::Ident(String::from("add")),
            Token::Assign(String::from("=")),
            Token::Function(String::from("fn")),
            Token::Lparen(String::from("(")),
            Token::Ident(String::from("x")),
            Token::Comma(String::from(",")),
            Token::Ident(String::from("y")),
            Token::Rparen(String::from(")")),
            Token::Lbrace(String::from("{")),
            Token::Ident(String::from("x")),
            Token::Plus(String::from("+")),
            Token::Ident(String::from("y")),
            Token::Semicolon(String::from(";")),
            Token::Rbrace(String::from("}")),
            Token::Semicolon(String::from(";")),

            Token::Bang(String::from("!")),
            Token::Minus(String::from("-")),
            Token::Slash(String::from("/")),
            Token::Asterisk(String::from("*")),
            Token::Int(String::from("5")),
            Token::Semicolon(String::from(";")),
            Token::Int(String::from("5")),
            Token::LT(String::from("<")),
            Token::Int(String::from("10")),
            Token::GT(String::from(">")),
            Token::Int(String::from("5")),
            Token::Semicolon(String::from(";")),

            Token::If(String::from("if")),
            Token::Lparen(String::from("(")),
            Token::Int(String::from("5")),
            Token::LT(String::from("<")),
            Token::Int(String::from("10")),
            Token::Rparen(String::from(")")),
            Token::Lbrace(String::from("{")),
            Token::Return(String::from("return")),
            Token::True(String::from("true")),
            Token::Semicolon(String::from(";")),
            Token::Rbrace(String::from("}")),
            Token::Else(String::from("else")),
            Token::Lbrace(String::from("{")),
            Token::Return(String::from("return")),
            Token::False(String::from("false")),
            Token::Semicolon(String::from(";")),
            Token::Rbrace(String::from("}")),

            Token::Int(String::from("10")),
            Token::Eq(String::from("==")),
            Token::Int(String::from("10")),
            Token::Semicolon(String::from(";")),

            Token::Int(String::from("10")),
            Token::NotEq(String::from("!=")),
            Token::Int(String::from("9")),
            Token::Semicolon(String::from(";")),

            Token::EOF(String::from("")),
        ];
        let lexer = Lexer::new(input);
        for (result, expected) in lexer.zip(output.iter()) {
            println!("Lexer: {:?} - {:?}", &result, expected);
            assert_eq!(&result, expected);
        }
    }
}
