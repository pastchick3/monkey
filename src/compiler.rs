use crate::code::Code;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::ast::Statement;
use crate::ast::Expression;
use crate::object::Object;
use crate::code::SymbolTable;
use crate::code::Symbol;

pub struct Compiler {
    input: Option<Vec<Statement>>,
    instructions: Vec<Code>,
    symbol_table: SymbolTable,
}

impl Compiler {
    pub fn new(parser: Parser, symbol_table: SymbolTable) -> Compiler {
        Compiler {
            input: Some(parser.collect()),
            instructions: vec!(),
            symbol_table,
        }
    }

    pub fn run(mut self) -> (Vec<Code>, SymbolTable) {
        let input = self.input.take().unwrap();
        for stmt in input.into_iter() {
            self.compile_statement(stmt);
        }
        (self.instructions, self.symbol_table)
    }

    fn compile_statement(&mut self, stmt: Statement) {
        match stmt {
            Statement::Let { ident, expr } => self.compile_let(ident, expr),
            Statement::Return(expr) => (),
            Statement::Expr(expr) => {
                self.compile_expression(expr);
                self.instructions.push(Code::Pop);
            },
            Statement::Block(block) => {
                for stmt in block.iter() {
                    self.compile_statement((**stmt).clone());
                }
            },
        }
    }

    fn compile_let(&mut self, ident: Expression, expr: Expression) {
        self.compile_expression(expr);
        let name = match ident {
            Expression::Ident(name) => name,
            ident => panic!("Invalid identifier {:?}.", ident),
        };
        let index = self.symbol_table.define(&name);
        self.instructions.push(Code::SetGlobal(index));
    }

    fn compile_expression(&mut self, expr: Expression) {
        match expr {
            Expression::Ident(v) => self.compile_ident(v),
            Expression::Int(v) => self.compile_int(v),
            Expression::Str(v) => self.instructions.push(Code::Constant(Object::Str(v))),
            Expression::Bool(v) => self.compile_bool(v),
            Expression::Array(exprs) => self.compile_array(exprs),
            Expression::Prefix { operator, expr } => self.compile_prefix(operator, *expr),
            Expression::Infix { operator, left, right } => self.compile_infix(operator, *left, *right),
            Expression::If { condition, consequence, alternative } => self.compile_if(*condition, *consequence, *alternative),
            Expression::Function { parameters, body } => (),
            Expression::Call { function, arguments } => (),
        }
    }

    fn compile_ident(&mut self, v: String) {
        match self.symbol_table.resolve(&v) {
            Some(Symbol { name: _, scope: _, index }) => self.instructions.push(Code::GetGlobal(*index)),
            None => panic!("Identifier {} not found.", v),
        };
    }

    fn compile_int(&mut self, v: String) {
        let int = Object::Int(i32::from_str_radix(&v, 10).unwrap());
        self.instructions.push(Code::Constant(int));
    }
    fn compile_bool(&mut self, v: String) {
        match v.as_str() {
            "true" => self.instructions.push(Code::True),
            "false" => self.instructions.push(Code::False),
            v => panic!("Invalid bool {}.", v),
        }
    }

    fn compile_array(&mut self, exprs: Vec<Box<Expression>>) {
        let size = exprs.len();
        for expr in exprs.into_iter() {
            self.compile_expression(*expr);
        }
        self.instructions.push(Code::Array(size));
    }

    fn compile_infix(&mut self, operator: String, left: Expression, right: Expression) {
        self.compile_expression(left);
        self.compile_expression(right);
        match operator.as_str() {
            "+" => self.instructions.push(Code::Add),
            "-" => self.instructions.push(Code::Sub),
            "*" => self.instructions.push(Code::Mul),
            "/" => self.instructions.push(Code::Div),
            "==" => self.instructions.push(Code::Equal),
            "!=" => self.instructions.push(Code::NotEqual),
            ">" => self.instructions.push(Code::GreaterThan),
            "<" => self.instructions.push(Code::LessThan),
            "[" => self.instructions.push(Code::Index),
            op => panic!("Unknown operator {}.", op),
        };
    }

    fn compile_prefix(&mut self, operator: String, expr: Expression) {
        self.compile_expression(expr);
        match operator.as_str() {
            "-" => self.instructions.push(Code::Minus),
            "!" => self.instructions.push(Code::Bang),
            op => panic!("Unknown operator {}.", op),
        };
    }

    fn compile_if(&mut self, condition: Expression, consequence: Statement, alternative: Statement) {
        self.compile_expression(condition);
        // consequence
        let pos = self.instructions.len();
        self.instructions.push(Code::JumpNotTruthy(9999));
        self.compile_statement(consequence);
        match self.instructions.pop().unwrap() {
            Code::Pop => (),
            code => self.instructions.push(code),
        }
        let offset = self.instructions.len() - pos;
        self.instructions.push(Code::JumpNotTruthy(offset));
        self.instructions.swap_remove(pos);
        // alternative
        let pos = self.instructions.len();
        self.instructions.push(Code::Jump(9999));
        self.compile_statement(alternative);
        match self.instructions.pop().unwrap() {
            Code::Pop => (),
            code => self.instructions.push(code),
        }
        let mut offset = self.instructions.len() - 1 - pos;
        if offset == 0 {
            offset = 1;
            self.instructions.push(Code::Null);
        };
        self.instructions.push(Code::Jump(offset));
        self.instructions.swap_remove(pos);
        
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn compiler() {
        let test_array = [
            ("1 + 2;", vec!(
                Code::Constant(Object::Int(1)),
                Code::Constant(Object::Int(2)),
                Code::Add,
                Code::Pop,
            )),
            ("1 - 2;", vec!(
                Code::Constant(Object::Int(1)),
                Code::Constant(Object::Int(2)),
                Code::Sub,
                Code::Pop,
            )),
            ("1 * 2;", vec!(
                Code::Constant(Object::Int(1)),
                Code::Constant(Object::Int(2)),
                Code::Mul,
                Code::Pop,
            )),
            ("1 / 2;", vec!(
                Code::Constant(Object::Int(1)),
                Code::Constant(Object::Int(2)),
                Code::Div,
                Code::Pop,
            )),
            ("1 == 2;", vec!(
                Code::Constant(Object::Int(1)),
                Code::Constant(Object::Int(2)),
                Code::Equal,
                Code::Pop,
            )),
            ("1 != 2;", vec!(
                Code::Constant(Object::Int(1)),
                Code::Constant(Object::Int(2)),
                Code::NotEqual,
                Code::Pop,
            )),
            ("1 > 2;", vec!(
                Code::Constant(Object::Int(1)),
                Code::Constant(Object::Int(2)),
                Code::GreaterThan,
                Code::Pop,
            )),
            ("1 < 2;", vec!(
                Code::Constant(Object::Int(1)),
                Code::Constant(Object::Int(2)),
                Code::LessThan,
                Code::Pop,
            )),
            ("-1;", vec!(
                Code::Constant(Object::Int(1)),
                Code::Minus,
                Code::Pop,
            )),
            ("!true;", vec!(
                Code::True,
                Code::Bang,
                Code::Pop,
            )),
            ("if (true) { 1 } else {2};", vec!(
                Code::True,
                Code::JumpNotTruthy(2),
                Code::Constant(Object::Int(1)),
                Code::Jump(1),
                Code::Constant(Object::Int(2)),
                Code::Pop,
            )),
            ("if (true) { 1 };", vec!(
                Code::True,
                Code::JumpNotTruthy(2),
                Code::Constant(Object::Int(1)),
                Code::Jump(1),
                Code::Null,
                Code::Pop,
            )),
            ("if (false) { 1 };", vec!(
                Code::False,
                Code::JumpNotTruthy(2),
                Code::Constant(Object::Int(1)),
                Code::Jump(1),
                Code::Null,
                Code::Pop,
            )),
            ("!(if (false) { 1 });", vec!(
                Code::False,
                Code::JumpNotTruthy(2),
                Code::Constant(Object::Int(1)),
                Code::Jump(1),
                Code::Null,
                Code::Bang,
                Code::Pop,
            )),
            ("
                let x = 5;
                if (x > 1) {
                    let y = x + 1;
                    y;
                }
            ", vec!(
                Code::Constant(Object::Int(5)),
                Code::SetGlobal(0),
                Code::GetGlobal(0),
                Code::Constant(Object::Int(1)),
                Code::GreaterThan,
                Code::JumpNotTruthy(6),
                Code::GetGlobal(0),
                Code::Constant(Object::Int(1)),
                Code::Add,
                Code::SetGlobal(1),
                Code::GetGlobal(1),
                Code::Jump(1),
                Code::Null,
                Code::Pop,
            )),
            ("\"a\" + \"b\";", vec!(
                Code::Constant(Object::Str(String::from("a"))),
                Code::Constant(Object::Str(String::from("b"))),
                Code::Add,
                Code::Pop,
            )),
            ("[1, 2][1];", vec!(
                Code::Constant(Object::Int(1)),
                Code::Constant(Object::Int(2)),
                Code::Array(2),
                Code::Constant(Object::Int(1)),
                Code::Index,
                Code::Pop,
            )),
        ];
        for (input, expected) in test_array.iter() {
            let lexer = Lexer::new(input);
            let parser = Parser::new(lexer);
            let symbol_table = SymbolTable::new();
            let compiler = Compiler::new(parser, symbol_table);
            let (output, symbol_table) = compiler.run();
            println!("Compiler: {:?} - {:?}", input, output);
            assert_eq!(expected, &output);
        }
    }
}
