use crate::ast::Expression;
use crate::ast::Statement;
use crate::parser::Parser;
use crate::object::Object;
use crate::object::Environment;

const TRUE: Object = Object::Bool(true);
const FALSE: Object = Object::Bool(false);
const NULL: Object = Object::Null;

pub struct Evaluator {
    input: Vec<Statement>,
    pos: usize,
    returned: bool,
    env: Environment,
}

impl Evaluator {
    pub fn new(parser: Parser, env: Environment) -> Evaluator {
        let input = parser.collect();
        Evaluator {
            input,
            pos: 0,
            returned: false,
            env,
        }
    }

    fn stmt(&self) -> Option<Statement> {
        if self.returned {
            return None;
        }
        if self.pos < self.input.len() {
            Some(self.input[self.pos].clone())
        } else {
            None
        }
    }

    fn forward(&mut self) -> () {
        self.pos += 1;
    }

    fn eval_statement(&mut self, stmt: Statement, env: &mut Environment) -> Object {
        match stmt {
            Statement::Expr(expr) => self.eval_expression(expr, env),
            Statement::Return(expr) => Object::Return(Box::new(self.eval_expression(expr, env))),
            Statement::Let { ident: Expression::Ident(ident), expr} => {
                let value = self.eval_expression(expr, env);
                env.set(ident, value);
                NULL
            },
            st => panic!("Invalid statement {:?}.", st),
        }
    }

    fn eval_block(&mut self, block: Statement, env: &mut Environment) -> Object {
        let block = match block {
            Statement::Block(v) => v,
            _ => panic!("Invalid block statement.")
        };
        let mut result = NULL;
        for stmt in block {
            result = self.eval_statement(*stmt, env);
            if let Object::Return(_) = result {
                return result;
            }
        }
        result
    }

    fn eval_expression(&mut self, expr: Expression, env: &mut Environment) -> Object {
        match expr {
            Expression::Int(v) => Object::Int(i32::from_str_radix(&v, 10).unwrap()),
            Expression::Bool(v) => if &v == "true" { TRUE } else { FALSE },
            Expression::Prefix { operator, expr } => self.eval_prefix(operator, *expr, env),
            Expression::Infix { operator, left, right } => self.eval_infix(operator, *left, *right, env),
            Expression::If { condition, consequence, alternative } => {
                self.eval_if(*condition, *consequence, *alternative, env)
            },
            Expression::Ident(ident) => match env.get(&ident) {
                Some(obj) => obj.clone(),
                None => panic!("Identifier {:?} not found.", ident),
            },
            Expression::Function { parameters, body } => Object::Function {
                parameters,
                body,
                env: env.clone(),
            },
            Expression::Call { function, arguments } => {
                self.eval_call(*function, arguments, env)
            },
        }
    }

    fn eval_prefix(&mut self, op: String, expr: Expression, env: &mut Environment) -> Object {
        let obj = self.eval_expression(expr, env);
        match op.as_str() {
            "!" => match obj {
                TRUE => FALSE,
                FALSE => TRUE,
                NULL => TRUE,
                _ => FALSE,
            },
            "-" => match obj {
                Object::Int(v) => Object::Int(-v),
                _ => panic!("Invalid prefix operand {:?}.", obj),
            },
            op => panic!("Invalid prefix operator {:?}.", op),
        }
    }

    fn eval_infix(&mut self, op: String, left: Expression, right: Expression,
                  env: &mut Environment) -> Object {
        let left = self.eval_expression(left, env);
        let right = self.eval_expression(right, env);
        if let Object::Int(l) = left {
            if let Object::Int(r) = right {
                match op.as_str() {
                    "+" => Object::Int(l+r),
                    "-" => Object::Int(l-r),
                    "*" => Object::Int(l*r),
                    "/" => Object::Int(l/r),
                    "<" => if l < r { TRUE } else { FALSE },
                    ">" => if l > r { TRUE } else { FALSE },
                    "==" => if l == r { TRUE } else { FALSE },
                    "!=" => if l != r { TRUE } else { FALSE },
                    op => panic!("unknown operator {:?}", op),
                }
            } else { panic!("type mismatch") }
        } else if let Object::Bool(l) = left {
            if let Object::Bool(r) = right {
                match op.as_str() {
                    "==" => if l == r { TRUE } else { FALSE },
                    "!=" => if l != r { TRUE } else { FALSE },
                    op => panic!("unknown operator {:?}", op),
                }
            } else { panic!("type mismatch") }
        } else { panic!("unexpected type") }
    }

    fn eval_if(&mut self, condition: Expression, consequence: Statement,
               alternative: Statement, env: &mut Environment) -> Object {
        let condition = self.eval_expression(condition, env);
        let block = match condition {
            NULL | FALSE => alternative,
            _ => consequence,
        };
        self.eval_block(block, env)
    }

    fn eval_call(&mut self, function: Expression, arguments: Vec<Box<Expression>>,
                 env: &mut Environment) -> Object {
        let function = self.eval_expression(function, env);
        if let Object::Function { parameters, body, env: fn_env } = function {
            let mut extended_fn_env = Environment::init(fn_env);
            for (par, aug) in parameters.into_iter().zip(arguments.into_iter()) {
                if let Expression::Ident(name) = *par {
                    extended_fn_env.set(name, self.eval_expression(*aug, env));
                } else {
                    panic!("Invalid parameter {:?}.", par);
                }
            }
            let result = self.eval_block(*body, &mut extended_fn_env);
            if let Object::Return(obj) = result {
                *obj
            } else {
                result
            }
        } else {
            panic!("Invalid function {:?}.", function);
        }
    }
}

impl Iterator for Evaluator {
    
    type Item = (Object, Environment);

    fn next(&mut self) -> Option<Self::Item> {
        match self.stmt() {
            Some(stmt) => {
                self.forward();
                // We cannot just pass self.env around, or there will be 2 mutable borrows of self.
                let mut env = self.env.clone();
                let result = self.eval_statement(stmt, &mut env);
                self.env = env;
                if let Object::Return(obj) = result {
                    self.returned = true;
                    Some((*obj, self.env.clone()))
                } else {
                    Some((result, self.env.clone()))
                }
            },
            None => None,
        }
    }
}


#[cfg(test)]
mod tests {

    use crate::lexer::Lexer;
    use super::Environment;
    use super::Expression;
    use super::Statement;
    use super::Parser;
    use super::Object;
    use super::Evaluator;

    #[test]
    fn evaluator() {
        let test_array = [
            ("5;", Object::Int(5), "5"),
            ("true;", Object::Bool(true), "true"),
            ("false;", Object::Bool(false), "false"),
            
            ("!true;", Object::Bool(false), "false"),
            ("!!5;", Object::Bool(true), "true"),

            ("-10;", Object::Int(-10), "-10"),

            ("2 + 1;", Object::Int(3), "3"),
            ("2 - 1;", Object::Int(1), "1"),
            ("2 * 1;", Object::Int(2), "2"),
            ("2 / 1;", Object::Int(2), "2"),
            ("1 + 2 * 3;", Object::Int(7), "7"),
            ("(1 + 2) * 3;", Object::Int(9), "9"),

            ("1 < 2;", Object::Bool(true), "true"),
            ("1 > 2;", Object::Bool(false), "false"),
            ("1 == 2;", Object::Bool(false), "false"),
            ("1 != 2;", Object::Bool(true), "true"),

            ("true == false;", Object::Bool(false), "false"),
            ("(1 < 2) != false;", Object::Bool(true), "true"),

            ("if (true) { 1 };", Object::Int(1), "1"),
            ("if (false) { 1 };", Object::Null, "Null"),
            ("if (1) { 1 };", Object::Int(1), "1"),
            ("if (0) { 1 };", Object::Int(1), "1"),
            ("if (1 < 2) { 1 } else { 2 };", Object::Int(1), "1"),
            ("if (1 > 2) { 1 } else { 2 };", Object::Int(2), "2"),

            ("return 10; 5;", Object::Int(10), "10"),

            ("let a = 5; a;", Object::Int(5), "5"),
            ("let a = 5; let b = a + 5; b;", Object::Int(10), "10"),

            ("fn() {};", Object::Function {
                parameters: Vec::new(),
                body: Box::new(Statement::Block(Vec::new())),
                env: Environment::new(),
            }, "function"),
            ("fn(x, y) { x };", Object::Function {
                parameters: vec!(
                    Box::new(Expression::Ident(String::from("x"))),
                    Box::new(Expression::Ident(String::from("y"))),
                ),
                body: Box::new(Statement::Block(vec!(
                    Box::new(Statement::Expr(Expression::Ident(String::from("x")))),
                ))),
                env: Environment::new(),
            }, "function"),

            ("let add = fn(x, y) { x + y;}; add(1, add(2, 3));", Object::Int(6), "6"),
            ("fn(x, y) { x + y;}(1, 2);", Object::Int(3), "3"),
        ];
        for (input, expected, display) in test_array.iter() {
            let env = Environment::new();
            let lexer = Lexer::new(input);
            let parser = Parser::new(lexer);
            let evaluator = Evaluator::new(parser, env);
            let output: Vec<_> = evaluator.collect();
            let obj = &output[output.len()-1].0;
            println!("Evaluator: {:?} - {:?}", input, obj);
            assert_eq!(expected, obj);
            assert_eq!(display, &format!("{}", obj));
        }
    }
}
