use std::collections::HashMap;

use crate::object::Object;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Code {
    Constant(Object),
    Pop,
    Add,
    Sub,
    Mul,
    Div,
    True,
    False,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    Minus,
    Bang,
    JumpNotTruthy(usize),
    Jump(usize),
    Null,
    SetGlobal(u32),
    GetGlobal(u32),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Scope {
    Global,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub scope: Scope,
    pub index: u32,
}

pub struct SymbolTable {
    map: HashMap<String, Symbol>,
    num_definitions: u32,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            map: HashMap::new(),
            num_definitions: 0,
        }
    }

    pub fn define(&mut self, name: &str) -> u32 {
        let index = self.num_definitions;
        self.num_definitions += 1;
        let symbol = Symbol {
            name: String::from(name),
            scope: Scope::Global,
            index,
        };
        self.map.insert(String::from(name), symbol);
        index
    }

    pub fn resolve(&mut self, name: &str) -> Option<&Symbol> {
        self.map.get(name)
    }
}