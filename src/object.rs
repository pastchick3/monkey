use std::collections::HashMap;
use std::fmt;

use crate::ast::Expression;
use crate::ast::Statement;
use crate::code::Code;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Object {
    Int(i32),
    Str(String),
    Bool(bool),
    Null,
    Return(Box<Object>),
    Array(Vec<Box<Object>>),
    Function {
        parameters: Vec<Box<Expression>>,
        body: Box<Statement>,
        env: Environment,
    },
    CompiledFunction {
        instructions: Vec<Code>,
        num_locals: usize,
        num_paras: usize,
    },
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Int(v) => write!(f, "{}", v),
            Object::Str(s) => write!(f, "{}", s),
            Object::Bool(v) => write!(f, "{}", v),
            Object::Null => write!(f, "Null"),
            Object::Return(obj) => write!(f, "{}", *obj),
            Object::Array(vec) => {
                let mut s = String::from("[");
                for obj in vec.iter() {
                    s += format!("{}, ", obj).as_str();
                }
                s.pop();
                s.pop();
                s += "]";
                write!(f, "{}", s)
            }
            Object::Function {
                parameters: _,
                body: _,
                env: _,
            } => write!(f, "function"),
            Object::CompiledFunction { instructions: _, num_locals: _, num_paras: _ } => write!(f, "compiled function"),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Environment {
    env: HashMap<String, Object>,
    outer: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            env: HashMap::new(),
            outer: None,
        }
    }

    pub fn init(outer: Environment) -> Environment {
        Environment {
            env: HashMap::new(),
            outer: Some(Box::new(outer)),
        }
    }

    pub fn get(&self, key: &String) -> Option<Object> {
        match self.env.get(key) {
            Some(value) => Some(value.clone()),
            None => match &self.outer {
                Some(e) => e.get(key),
                None => panic!("Identifier {} not found.", key),
            },
        }
    }

    pub fn set(&mut self, key: String, value: Object) -> () {
        self.env.insert(key, value);
    }
}
