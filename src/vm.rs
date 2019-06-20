use std::collections::HashMap;

use crate::code::Code;
use crate::code::SymbolTable;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::compiler::Compiler;
use crate::object::Object;

const TRUE: Object = Object::Bool(true);
const FALSE: Object = Object::Bool(false);
const NULL: Object = Object::Null;

#[derive(Clone)]
struct Frame {
    instructions: Vec<Code>,
    base: usize,
}

pub struct VM {
    frames: Vec<Frame>,
    instructions: Vec<Code>,
    stack: Vec<Object>,
    base: usize,
    last_popped: Option<Object>,
    jump: usize,
    globals: HashMap<usize, Object>,
}

impl VM {
    pub fn new(mut instructions: Vec<Code>, globals: HashMap<usize, Object>) -> VM {
        instructions.reverse();
        VM {
            frames: vec!(),
            instructions,
            stack: vec!(),
            base: 0,
            last_popped: None,
            jump: 0,
            globals,
        }
    }

    pub fn run(mut self) -> (Object, Option<Object>, HashMap<usize, Object>) {
        loop {
            match self.instructions.pop() {
                Some(code) => {
                    if self.jump == 0 {
                        self.execute(code);
                    } else {
                        self.jump -= 1;
                    };
                },
                None => break,
            };
        };
        match self.stack.pop() {
            Some(obj) => (obj, self.last_popped, self.globals),
            None => (NULL, self.last_popped, self.globals),
        }
    }

    fn execute(&mut self, code: Code) {
        match code {
            Code::Constant(obj) => self.stack.push(obj),
            op @ Code::Add | op @ Code::Sub | op @ Code::Mul | op @ Code::Div => self.execute_arithmetic(op),
            op @ Code::Equal | op @ Code::NotEqual | op @ Code::GreaterThan | op @ Code::LessThan => self.execute_comparison(op),
            Code::True => self.stack.push(TRUE),
            Code::False => self.stack.push(FALSE),
            op @ Code::Minus | op @ Code::Bang => self.execute_prefix(op),
            Code::Pop => { self.last_popped = self.stack.pop(); },
            Code::JumpNotTruthy(offset) => self.execute_jump_not_truthy(offset),
            Code::Jump(offset) => self.execute_jump(offset),
            Code::Null => self.stack.push(NULL),
            Code::SetGlobal(index) => { self.globals.insert(index, self.stack.pop().unwrap()); },
            Code::GetGlobal(index) => { self.stack.push(self.globals.get(&index).unwrap().clone()); },
            Code::Array(size) => self.execute_array(size),
            Code::Index => self.execute_index(),
            Code::ReturnValue => self.execute_return_value(),
            Code::Return => self.execute_return(),
            Code::Call(num_args) => self.execute_call(num_args),
            Code::SetLocal(index) => { self.stack.swap_remove(self.base+index); },
            Code::GetLocal(index) => { self.stack.push(self.stack.get(self.base+index).unwrap().clone()); },
        }
    }

    fn push_frame(&mut self, mut instructions: Vec<Code>, base: usize) {
        self.frames.push(Frame {
            instructions: self.instructions.clone(),
            base,
        });
        instructions.reverse();
        self.instructions = instructions;
        self.base = base;
    }

    fn pop_frame(&mut self) {
        let Frame { instructions, base } = self.frames.pop().unwrap();
        self.instructions = instructions;
        while self.stack.len() > base {
            self.stack.pop();
        }
    }

    fn execute_arithmetic(&mut self, op: Code) {
        let right = self.stack.pop().unwrap();
        if let Object::Int(right) = right {
            let left = self.stack.pop().unwrap();
            if let Object::Int(left) = left {
                let value = match op {
                    Code::Add => left + right,
                    Code::Sub => left - right,
                    Code::Mul => left * right,
                    Code::Div => left / right,
                    op => panic!("Unexpected arithmatic operator {:?}.", op),
                };
                self.stack.push(Object::Int(value));
            } else {
                panic!("Expect Object::Int, get {}.", left);
            };
        } else if let Object::Str(right) = right {
            let left = self.stack.pop().unwrap();
            if let Object::Str(left) = left {
                let value = match op {
                    Code::Add => left + &right,
                    op => panic!("Unexpected arithmatic operator {:?}.", op),
                };
                self.stack.push(Object::Str(value));
            } else {
                panic!("Expect Object::Str, get {}.", left);
            };
        } else {
            panic!("Expect Object::Int or Object::Str, get {}.", right);
        };
    }

    fn execute_comparison(&mut self, op: Code) {
        let obj_right = self.stack.pop().unwrap();
        if let Object::Int(right) = obj_right {
            let obj_left = self.stack.pop().unwrap();
            if let Object::Int(left) = obj_left {
                match op {
                    Code::Equal => self.stack.push(Object::Bool(left==right)),
                    Code::NotEqual => self.stack.push(Object::Bool(left!=right)),
                    Code::GreaterThan => self.stack.push(Object::Bool(left>right)),
                    Code::LessThan => self.stack.push(Object::Bool(left<right)),
                    op => panic!("Unknown operator {:?}.", op),
                }
            } else {
                panic!("Expect Object::Int, get {}.", obj_left);
            };
        } else if let Object::Bool(right) = obj_right {
            let obj_left = self.stack.pop().unwrap();
            if let Object::Bool(left) = obj_left {
                match op {
                    Code::Equal => self.stack.push(Object::Bool(left==right)),
                    Code::NotEqual => self.stack.push(Object::Bool(left!=right)),
                    op => panic!("Unknown operator {:?}.", op),
                }
            } else {
                panic!("Expect Object::Bool, get {}.", obj_left);
            };
        } else {
            panic!("Expect Object::Bool or Object::Int, get {}.", obj_right);
        };
    }

    fn execute_prefix(&mut self, operator: Code) {
        match operator {
            Code::Minus => {
                match self.stack.pop().unwrap() {
                    Object::Int(v) => self.stack.push(Object::Int(-v)),
                    obj => panic!("Expect Object::Int, get {:?}.", obj),
                };
            },
            Code::Bang => {
                match self.stack.pop().unwrap() {
                    Object::Bool(v) => self.stack.push(Object::Bool(!v)),
                    NULL => self.stack.push(Object::Bool(true)),
                    obj => panic!("Expect Object::Bool, get {:?}.", obj),
                };
            },
            _ => (),
        }
    }

    fn execute_jump_not_truthy(&mut self, offset: usize) {
        match self.stack.pop().unwrap() {
            Object::Bool(false) | NULL => self.execute_jump(offset),
            _ => (),
        }
    }

    fn execute_jump(&mut self, offset: usize) {
        self.jump = offset;
    }

    fn execute_array(&mut self, size: usize) {
        let mut array = Vec::new();
        for _ in 0..size {
            array.push(Box::new(self.stack.pop().unwrap()));
        }
        array.reverse();
        self.stack.push(Object::Array(array));
    }

    fn execute_index(&mut self) {
        let index = match self.stack.pop().unwrap() {
            Object::Int(v) => v,
            obj => panic!("Expect Object::Int, get {:?}.", obj),
        };
        let array = match self.stack.pop().unwrap() {
            Object::Array(v) => v,
            obj => panic!("Expect Object::Array, get {:?}.", obj),
        };
        self.stack.push(match array.get(index as usize) {
            Some(obj) => (**obj).clone(),
            None => NULL,
        });
    }

    fn execute_call(&mut self, num_args: usize) {
        let func = self.stack.remove(self.stack.len()-num_args-1);
        let (instructions, num_locals, num_paras) = match func {
            Object::CompiledFunction { instructions, num_locals, num_paras } => (instructions, num_locals, num_paras),
            obj => panic!("Expect Object::CompiledFunction, get {:?}.", obj),
        };
        assert_eq!(num_args, num_paras, "{} args vs {} paras", num_args, num_paras);
        self.push_frame(instructions, self.stack.len()-num_args);
        for _ in 0..num_locals {
            self.stack.push(NULL);
        }
    }

    fn execute_return_value(&mut self) {
        let value = self.stack.pop().unwrap();
        self.pop_frame();
        self.stack.push(value);
    }

    fn execute_return(&mut self) {
        self.pop_frame();
        self.stack.push(NULL);
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn vm() {
        let test_array = [
            ("1 + 2;", NULL, Some(Object::Int(3))),
            ("1 - 2;", NULL, Some(Object::Int(-1))),
            ("1 * 2;", NULL, Some(Object::Int(2))),
            ("1 / 2;", NULL, Some(Object::Int(0))),
            ("1 == 2;", NULL, Some(Object::Bool(false))),
            ("1 != 2;", NULL, Some(Object::Bool(true))),
            ("1 > 2;", NULL, Some(Object::Bool(false))),
            ("1 < 2;", NULL, Some(Object::Bool(true))),
            ("true == true;", NULL, Some(Object::Bool(true))),
            ("true != true;", NULL, Some(Object::Bool(false))),
            ("-1;", NULL, Some(Object::Int(-1))),
            ("!true;", NULL, Some(Object::Bool(false))),
            ("!(if (false) { 1 });", NULL, Some(Object::Bool(true))),
            ("if (true) { 1 } else {2};", NULL, Some(Object::Int(1))),
            ("if (false) { 1 };", NULL, Some(NULL)),
            ("let a = 1; a + 1;", NULL, Some(Object::Int(2))),
            ("\"a\" + \"b\";", NULL, Some(Object::Str(String::from("ab")))),
            ("[1, 2];", NULL, Some(Object::Array(vec!(
                Box::new(Object::Int(1)),
                Box::new(Object::Int(2)),
            )))),
            ("[1, 2][1];", NULL, Some(Object::Int(2))),
            ("fn() { return 1; }();", NULL, Some(Object::Int(1))),
            ("fn() { 1; }();", NULL, Some(Object::Int(1))),
            ("fn() {}();", NULL, Some(NULL)),
            ("
                let a = 1; 
                let b = fn() { let a = 2; a; }();
                a + b;
            ", NULL, Some(Object::Int(3))),
            ("fn(a) { a; }(1);", NULL, Some(Object::Int(1))),
        ];
        for (input, result, popped) in test_array.iter() {
            let lexer = Lexer::new(input);
            let parser = Parser::new(lexer);
            let symbol_table = SymbolTable::new(None);
            let compiler = Compiler::new(parser, symbol_table);
            let (code, _symbol_table) = compiler.run();
            let globals = HashMap::new();
            let vm = VM::new(code, globals);
            let (r, p, _g) = vm.run();
            println!("VM: {:?} - {:?} - {:?}", input, r, p);
            assert_eq!(result, &r);
            assert_eq!(popped, &p);
        }
    }
}
