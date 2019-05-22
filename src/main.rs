mod lexer;

use lexer::Lexer;
use std::io;
use std::io::Write;

fn main() {
    let prompt = ">> ";
    let mut input = String::new();
    
    println!("Welcome to the Monkey Programming language in Rust!");
    loop {
        print!("{}", prompt);
        io::stdout().flush().unwrap(); 
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                for token in Lexer::new(&input) {
                    println!("{:?}", token);
                }
            }
            Err(_) => continue,
        }
    }
}
