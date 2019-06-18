use crate::code::Code;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::compiler::Compiler;
use crate::object::Object;

const TRUE: Object = Object::Bool(true);
const FALSE: Object = Object::Bool(false);

pub struct VM {
    instructions: Option<Vec<Code>>,
    stack: Vec<Object>,
    last_popped: Option<Object>,
}

impl VM {
    pub fn new(instructions: Vec<Code>) -> VM {
        VM {
            instructions: Some(instructions),
            stack: vec!(),
            last_popped: None,
        }
    }

    pub fn run(mut self) -> (Option<Object>, Object) {
        let instructions = self.instructions.take().unwrap();
        for code in instructions.into_iter() {
            self.execute(code);
        }
        match self.stack.pop() {
            Some(obj) => (self.last_popped, obj),
            None => (self.last_popped, Object::Null),
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
        }
    }

    fn execute_arithmetic(&mut self, op: Code) {
        let right = match self.stack.pop().unwrap() {
            Object::Int(right) => right,
            obj => panic!("Expected Object::Int, get {:?}.", obj),
        };
        let left = match self.stack.pop().unwrap() {
            Object::Int(left) => left,
            obj => panic!("Expected Object::Int, get {:?}.", obj),
        };
        let int = match op {
            Code::Add => left + right,
            Code::Sub => left - right,
            Code::Mul => left * right,
            Code::Div => left / right,
            op => panic!("Unexpected arithmatic operator {:?}.", op),
        };
        self.stack.push(Object::Int(int));
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
                    obj => panic!("Expect Object::Bool, get {:?}.", obj),
                };
            },
            _ => (),
        }
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn vm() {
        let test_array = [
            ("1 + 2;", Object::Null, Some(Object::Int(3))),
            ("1 - 2;", Object::Null, Some(Object::Int(-1))),
            ("1 * 2;", Object::Null, Some(Object::Int(2))),
            ("1 / 2;", Object::Null, Some(Object::Int(0))),
            ("1 == 2;", Object::Null, Some(Object::Bool(false))),
            ("1 != 2;", Object::Null, Some(Object::Bool(true))),
            ("1 > 2;", Object::Null, Some(Object::Bool(false))),
            ("1 < 2;", Object::Null, Some(Object::Bool(true))),
            ("true == true;", Object::Null, Some(Object::Bool(true))),
            ("true != true;", Object::Null, Some(Object::Bool(false))),
            ("-1;", Object::Null, Some(Object::Int(-1))),
            ("!true;", Object::Null, Some(Object::Bool(false))),
        ];
        for (input, result, popped) in test_array.iter() {
            let lexer = Lexer::new(input);
            let parser = Parser::new(lexer);
            let compiler = Compiler::new(parser);
            let code = compiler.run();
            let vm = VM::new(code);
            let (p, r) = vm.run();
            println!("VM: {:?} - {:?} - {:?}", input, r, p);
            assert_eq!(result, &r);
            assert_eq!(popped, &p);
        }
    }
}
