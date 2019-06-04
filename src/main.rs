mod token;
mod lexer;
mod ast;
mod parser;
mod object;
mod evaluator;

use lexer::Lexer;
use parser::Parser;
use evaluator::Evaluator;
use object::Environment;
use std::io;
use std::io::Write;

fn main() {
    println!("Welcome to the Monkey Programming language in Rust!");
    let mut environment = Environment::new();
    loop {
        print!(">> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let lexer = Lexer::new(&input);
                let parser = Parser::new(lexer);
                let evaluator = Evaluator::new(parser, environment.clone());
                for (obj, env) in evaluator {
                    println!("{}", obj);
                    environment = env;
                }
                
            }
            Err(_) => continue,
        }
    }
}
