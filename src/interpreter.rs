use crate::parser::{Literals, Node};
use std::{cell::RefCell, collections::HashMap, ops::Deref, rc::Rc};

macro_rules! compare {
    ($left:expr, $condition:expr, $right:expr) => {
        match $condition.as_str() {
            "==" => $left == $right,
            "!=" => $left != $right,
            ">" => $left > $right,
            ">=" => $left >= $right,
            "<" => $left < $right,
            "<=" => $left <= $right,
            _ => unreachable!()
        }
    };
}

#[derive(Debug)]
pub enum ErrorTypes {
    UnknownError,
    TypeError,
    UndefinedVar,
    UndefinedFn,
}

#[derive(Debug)]
pub struct InterpreterError {
    pub r#type: ErrorTypes,
    pub message: String
}

type InterpreterResult<T> = Result<T, InterpreterError>;

#[derive(Debug, Clone)]
enum Values {
    Integer(i64),
    String(String),
    Boolean(bool),
    Function {
        args: Vec<Box<Node>>,
        scope: Box<Node>
    },
    None,
}

impl Values {
    fn is_none(&self) -> bool { matches!(self, Values::None) }
}

struct Env {
    vars: HashMap<String, Values>,
    parent: Option<Rc<RefCell<Env>>>
}

impl Env {
    fn new(parent: Option<Rc<RefCell<Env>>>) -> Self {
        Env {
            vars: HashMap::new(),
            parent
        }
    }

    fn set(&mut self, name: &str, value: Values) {
        self.vars.insert(name.to_string(), value);
    }

    fn get(&self, name: &str) -> InterpreterResult<Values> {
        if let Some(value) = self.vars.get(name) {
            return Ok(value.clone());
        }

        if let Some(ref parent) = self.parent {
            return parent.borrow().get(name);
        }

        Err(InterpreterError {
            r#type: ErrorTypes::UndefinedVar,
            message: format!("Cannot find var: {:?}", name)
        })
    }
}

pub struct Interpreter {
    env: Rc<RefCell<Env>>
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Rc::new(RefCell::new(Env::new(None)))
        }
    }

    fn handle_fn(&mut self, identifier: &Box<Node>, args: &Vec<Box<Node>>, scope: &Box<Node>) -> InterpreterResult<Values> {
        let name = match identifier.deref() {
            Node::Identifier(identifier) => identifier,
            _ => unreachable!()
        };

        let function = Values::Function {
            args: args.clone(),
            scope: scope.clone()
        };

        self.env.borrow_mut().set(name.as_str(), function);

        Ok(Values::None)
    }

    fn handle_ret(&mut self, node: &Box<Node>) -> InterpreterResult<Values> {
        let value = match node.deref() {
            Node::Literal(literal) => match literal {
                Literals::Int(integer) => Values::Integer(integer.clone()),
                Literals::String(str) => Values::String(str.clone()),
                Literals::Boolean(boolean) => Values::Boolean(boolean.clone()),
            },
            Node::Identifier(name) => self.env.borrow().get(name.as_str())?,
            Node::FunctionCall { identifier, args } => self.handle_fn_call(identifier, args)?,
            _ => return Err(InterpreterError {
                r#type: ErrorTypes::UnknownError,
                message: format!("Something went wrong while running handle_ret")
            })
        };

        Ok(value)
    }

    fn handle_fn_call(&mut self, identifier: &Box<Node>, args: &Vec<Box<Node>>) -> InterpreterResult<Values> {
        let name = match identifier.deref() {
            Node::Identifier(identifier) => identifier,
            _ => unreachable!()
        };

        let (fn_args, fn_scope) = match self.env.borrow().get(name.as_str()) {
            Ok(Values::Function { args, scope, .. }) => (args, scope),
            _ => return Err(InterpreterError {
                r#type: ErrorTypes::UndefinedFn,
                message: format!("Cannot find function: {:?}", name)
            })
        };

        if args.len() != fn_args.len() {
            return Err(InterpreterError {
                r#type: ErrorTypes::TypeError,
                message: format!("Argument mismatch on function {:?}, Expected {} but found only {}", name, fn_args.len(), args.len())
            });
        }

        let fn_env = Rc::new(RefCell::new(Env::new(None)));

        for (fn_arg, arg) in fn_args.deref().into_iter().zip(args.deref().into_iter()) {
            if let Node::Identifier(fn_arg) = fn_arg.deref() {
                let val = match arg.deref() {
                    Node::Literal(Literals::Int(integer)) => Values::Integer(integer.clone()),
                    Node::Literal(Literals::String(str)) => Values::String(str.clone()),
                    Node::Literal(Literals::Boolean(boolean)) => Values::Boolean(boolean.clone()),
                    Node::Identifier(identifier) => self.env.borrow().get(identifier.as_str())?,
                    Node::FunctionCall { identifier, args } => self.handle_fn_call(identifier, args)?,
                    _ => return Err(InterpreterError {
                        r#type: ErrorTypes::UnknownError,
                        message: format!("Something went wrong while running handle_fn_call")
                    })
                };

                fn_env.borrow_mut().set(fn_arg, val);
            }
        }

        let prev_env = std::mem::replace(&mut self.env, fn_env);

        if let Node::Scope { body } = fn_scope.deref() {
            for scope_node in body {
                let ret_value = self.exec_node(scope_node.deref())?;
                if !ret_value.is_none() {
                    return Ok(ret_value);
                }
            }
        }

        self.env = prev_env;

        Ok(Values::None)
    }

    fn handle_scope(&mut self, body: &Vec<Box<Node>>) -> InterpreterResult<Values> {
        let new_env = Rc::new(RefCell::new(Env::new(Some(self.env.clone()))));
        let prev_env = std::mem::replace(&mut self.env, new_env);

        for scope_node in body {
            self.exec_node(scope_node.deref())?;
        }

        self.env = prev_env;

        Ok(Values::None)
    }

    fn handle_var(&mut self, identifier: &Box<Node>, value: &Box<Node>) -> InterpreterResult<Values> {
        let name = match identifier.deref() {
            Node::Identifier(identifier) => identifier,
            _ => unreachable!()
        };

        let val = match value.deref() {
            Node::Literal(Literals::Int(integer)) => Values::Integer(integer.clone()),
            Node::Literal(Literals::String(str)) => Values::String(str.clone()),
            Node::Literal(Literals::Boolean(boolean)) => Values::Boolean(boolean.clone()),
            Node::Identifier(identifier) => self.env.borrow().get(identifier.as_str())?,
            Node::FunctionCall { identifier, args } => self.handle_fn_call(identifier, args)?,
            _ => return Err(InterpreterError {
                r#type: ErrorTypes::TypeError,
                message: format!("Unknown Type on variable {:?}", name)
            })
        };

        self.env.borrow_mut().set(name.as_str(), val);

        Ok(Values::None)
    }

    fn handle_update(&mut self, identifier: &Box<Node>, value: &Box<Node>) -> InterpreterResult<Values> {
        let name = match identifier.deref() {
            Node::Identifier(identifier) => identifier,
            _ => unreachable!()
        };

        let val = match value.deref() {
            Node::Literal(Literals::Int(integer)) => Values::Integer(integer.clone()),
            Node::Literal(Literals::String(str)) => Values::String(str.clone()),
            Node::Literal(Literals::Boolean(boolean)) => Values::Boolean(boolean.clone()),
            Node::Identifier(identifier) => self.env.borrow().get(identifier.as_str())?,
            Node::FunctionCall { identifier, args } => self.handle_fn_call(identifier, args)?,
            _ => return Err(InterpreterError {
                r#type: ErrorTypes::TypeError,
                message: format!("Unknown Type on variable {:?}", name)
            })
        };

        self.env.borrow_mut().set(name.as_str(), val);

        Ok(Values::None)
    }

    fn handle_log(&mut self, log_type: &str, args: &Vec<Box<Node>>) -> InterpreterResult<Values> {
        let mut output = String::new();

        for arg in args {
            match arg.deref() {
                Node::Literal(Literals::String(value)) => { output.push_str(value.as_str()); },
                Node::Literal(Literals::Int(value)) => { output.push_str(value.to_string().as_str()); },
                Node::Literal(Literals::Boolean(value)) => { output.push_str(value.to_string().as_str()); },
                Node::Identifier(name) => {
                    let value = self.env.borrow().get(name.as_str())?;

                    match value {
                        Values::Integer(integer) => { output.push_str(integer.to_string().as_str()); },
                        Values::String(str) => { output.push_str(str.as_str()); },
                        Values::Boolean(boolean) => { output.push_str(boolean.to_string().as_str()); }
                        _ => {}
                    }
                },
                Node::FunctionCall { identifier, args } => {
                    let ret_value = self.handle_fn_call(identifier, args)?;

                    match ret_value {
                        Values::Integer(integer) => { output.push_str(integer.to_string().as_str()); },
                        Values::String(str) => { output.push_str(str.as_str()); },
                        Values::Boolean(boolean) => { output.push_str(boolean.to_string().as_str()); }
                        _ => {}
                    }
                }
                _ => ()
            }
        }

        match log_type {
            "log" => { print!("{output}"); },
            "logl" => { println!("{output}"); }
            _ => ()
        }

        Ok(Values::None)
    }

    fn handle_check(&mut self, condition: &Box<Node>, scope: &Box<Node>) -> InterpreterResult<Values> {
        let condition = match condition.deref() {
            Node::Condition { left, condition, right } => {
                let left_value = match left.deref() {
                    Node::Literal(Literals::Int(integer)) => Values::Integer(*integer),
                    Node::Literal(Literals::String(str)) => Values::String(str.clone()),
                    Node::Literal(Literals::Boolean(boolean)) => Values::Boolean(*boolean),
                    Node::Identifier(identifier) => self.env.borrow().get(identifier)?,
                    _ => return Err(InterpreterError {
                        r#type: ErrorTypes::TypeError,
                        message: format!("Unknown type on the left side of the condition")
                    })
                };

                let right_value = match right.deref() {
                    Node::Literal(Literals::Int(integer)) => Values::Integer(*integer),
                    Node::Literal(Literals::String(str)) => Values::String(str.clone()),
                    Node::Literal(Literals::Boolean(boolean)) => Values::Boolean(*boolean),
                    Node::Identifier(identifier) => self.env.borrow().get(identifier)?,
                    _ => return Err(InterpreterError {
                        r#type: ErrorTypes::TypeError,
                        message: format!("Unknown type on the right side of the condition")
                    })
                };

                match (left_value, right_value) {
                    (Values::Integer(left_int), Values::Integer(right_int)) => compare!(left_int, condition, right_int),
                    (Values::String(left_str), Values::String(right_str)) => compare!(left_str, condition, right_str),
                    (Values::Boolean(left_boolean), Values::Boolean(right_boolean)) => compare!(left_boolean, condition, right_boolean),
                    _ => return Err(InterpreterError {
                        r#type: ErrorTypes::TypeError,
                        message: format!("Cannot compare {:?} to {:?}", left, right)
                    })
                }
            },
            Node::Literal(literal) => match literal {
                Literals::Int(integer) => *integer > 0,
                Literals::String(str) => str.len() > 0,
                Literals::Boolean(boolean) => *boolean
            },
            _ => return Err(InterpreterError {
                r#type: ErrorTypes::UnknownError,
                message: format!("Something went wrong in handle_check")
            })
        };

        let new_env = Rc::new(RefCell::new(Env::new(Some(self.env.clone()))));
        let prev_env = std::mem::replace(&mut self.env, new_env);

        if condition {
            if let Node::Scope { body } = scope.deref() {
                for scope_node in body {
                    let ret_value = self.exec_node(scope_node.deref())?;
                    if !ret_value.is_none() {
                        return Ok(ret_value);
                    }
                }
            }
        }

        self.env = prev_env;

        Ok(Values::None)
    }

    fn handle_while(&mut self, condition: &Box<Node>, scope: &Box<Node>) -> InterpreterResult<Values> {
        let condition = match condition.deref() {
            Node::Condition { left, condition, right } => {
                let left_value = match left.deref() {
                    Node::Literal(Literals::Int(integer)) => Values::Integer(*integer),
                    Node::Literal(Literals::String(str)) => Values::String(str.clone()),
                    Node::Literal(Literals::Boolean(boolean)) => Values::Boolean(*boolean),
                    Node::Identifier(identifier) => self.env.borrow().get(identifier)?,
                    _ => return Err(InterpreterError {
                        r#type: ErrorTypes::TypeError,
                        message: format!("Unknown type on the left side of the condition")
                    })
                };

                let right_value = match right.deref() {
                    Node::Literal(Literals::Int(integer)) => Values::Integer(*integer),
                    Node::Literal(Literals::String(str)) => Values::String(str.clone()),
                    Node::Literal(Literals::Boolean(boolean)) => Values::Boolean(*boolean),
                    Node::Identifier(identifier) => self.env.borrow().get(identifier)?,
                    _ => return Err(InterpreterError {
                        r#type: ErrorTypes::TypeError,
                        message: format!("Unknown type on the right side of the condition")
                    })
                };

                match (left_value, right_value) {
                    (Values::Integer(left_int), Values::Integer(right_int)) => compare!(left_int, condition, right_int),
                    (Values::String(left_str), Values::String(right_str)) => compare!(left_str, condition, right_str),
                    (Values::Boolean(left_boolean), Values::Boolean(right_boolean)) => compare!(left_boolean, condition, right_boolean),
                    _ => return Err(InterpreterError {
                        r#type: ErrorTypes::TypeError,
                        message: format!("Cannot compare {:?} to {:?}", left, right)
                    })
                }
            },
            Node::Literal(literal) => match literal {
                Literals::Int(integer) => *integer > 0,
                Literals::String(str) => str.len() > 0,
                Literals::Boolean(boolean) => *boolean
            },
            _ => return Err(InterpreterError {
                r#type: ErrorTypes::UnknownError,
                message: format!("Something went wrong in handle_while")
            })
        };

        let new_env = Rc::new(RefCell::new(Env::new(Some(self.env.clone()))));
        let prev_env = std::mem::replace(&mut self.env, new_env);

        while condition {
            if let Node::Scope { body } = scope.deref() {
                for scope_node in body {
                    let ret_value = self.exec_node(scope_node.deref())?;
                    if !ret_value.is_none() {
                        return Ok(ret_value);
                    }
                }
            }
        }

        self.env = prev_env;

        Ok(Values::None)
    }

    fn exec_node(&mut self, node: &Node) -> InterpreterResult<Values> {
        match node {
            Node::Function { identifier, args, scope } => self.handle_fn(identifier, args, scope),
            Node::FunctionCall { identifier, args } => self.handle_fn_call(identifier, args),
            Node::Return(value) => self.handle_ret(value),
            Node::Scope { body } => self.handle_scope(body),
            Node::Var { identifier, value } => self.handle_var(identifier, value),
            Node::Update { identifier, value } => self.handle_update(identifier, value),
            Node::Check { condition, scope } => self.handle_check(condition, scope),
            Node::While { condition, scope } => self.handle_while(condition, scope),
            Node::Log { r#type, args } => self.handle_log(r#type.as_str(), args),
            _ => Ok(Values::None)
        }
    }

    pub fn run(&mut self, ast: &Vec<Node>) -> InterpreterResult<()> {
        for node in ast {
            self.exec_node(node)?;
        }

        Ok(())
    }
}
