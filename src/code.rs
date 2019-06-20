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
    SetGlobal(usize),
    GetGlobal(usize),
    Array(usize),
    Index,
    ReturnValue,
    Return,
    Call(usize),
    SetLocal(usize),
    GetLocal(usize),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Scope {
    Global,
    Local,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub scope: Scope,
    pub index: usize,
}

#[derive(Clone)]
pub struct SymbolTable {
    pub outer: Option<Box<SymbolTable>>,
    pub map: HashMap<String, Symbol>,
    pub num_definitions: usize,
}

impl SymbolTable {
    pub fn new(outer: Option<Box<SymbolTable>>) -> SymbolTable {
        SymbolTable {
            outer,
            map: HashMap::new(),
            num_definitions: 0,
        }
    }

    pub fn get_outer(mut self) -> Option<Box<SymbolTable>> {
        self.outer.take()
    }

    pub fn define(&mut self, name: &str) -> Symbol {
        let index = self.num_definitions;
        self.num_definitions += 1;
        let symbol = Symbol {
            name: String::from(name),
            scope: match self.outer {
                Some(_) => Scope::Local,
                None => Scope::Global,
            },
            index,
        };
        self.map.insert(String::from(name), symbol.clone());
        symbol
    }

    pub fn resolve(&self, name: &str) -> Option<Symbol> {
        if let Some(sym) = self.map.get(name) {
            Some(sym.clone())
        } else if let Some(outer) = &self.outer {
            outer.resolve(name)
        } else {
            None
        }
    }
}