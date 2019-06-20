mod token;
mod lexer;
mod ast;
mod parser;
mod object;
mod evaluator;

mod code;
mod compiler;
mod vm;

use lexer::Lexer;
use parser::Parser;
use evaluator::Evaluator;
use object::Environment;
use compiler::Compiler;
use code::SymbolTable;
use vm::VM;
use std::io;
use std::io::Write;
use std::env;
use std::collections::HashMap;

fn main() {
    let args: Vec<String> = env::args().collect();
    let vm_flag = if args.len() > 1 && args[1].as_str() == "vm" {
        true
    } else {
        false
    };
    println!("Welcome to the Monkey Programming Language in Rust! ({})",
             if vm_flag { "VM" } else { "Interpreter" });
    let mut environment = Environment::new();
    let mut symbol_table = SymbolTable::new(None);
    let mut globals = HashMap::new();
    loop {
        print!(">> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let lexer = Lexer::new(&input);
                let parser = Parser::new(lexer);
                if vm_flag {
                    let compiler = Compiler::new(parser, symbol_table);
                    let (code, sym_table) = compiler.run();
                    let vm = VM::new(code, globals);
                    let (result, _popped, gb) = vm.run();
                    println!("{}", result);
                    symbol_table = sym_table;
                    globals = gb;
                } else {
                    let evaluator = Evaluator::new(parser, environment.clone());
                    for (obj, env) in evaluator {
                        println!("{}", obj);
                        environment = env;
                    }
                }
            }
            Err(_) => continue,
        }
    }
}
