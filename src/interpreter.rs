use rand::Rng;

use crate::parser::{Literals, Node};
use std::{cell::RefCell, collections::HashMap, mem::discriminant, ops::Deref, path::PathBuf, rc::Rc, usize};

macro_rules! compare {
    ($left:expr, $condition:expr, $right:expr) => {
        match $condition.as_str() {
            "=="    => $left == $right,
            "!="    => $left != $right,
            ">"     => $left > $right,
            ">="    => $left >= $right,
            "<"     => $left < $right,
            "<="    => $left <= $right,
            _       => unreachable!(),
        }
    };
}

#[derive(Debug)]
pub enum ErrorTypes {
    IndexOutOfBounds,
    MathError,
    UnknownError,
    TypeError,
    UndefinedVar,
    UndefinedFn,
}

#[derive(Debug)]
pub struct InterpreterError {
    pub r#type: ErrorTypes,
    pub message: String,
}

type InterpreterResult<T> = Result<T, InterpreterError>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Values {
    Integer(i64),
    String(String),
    Boolean(bool),
    Array(Vec<Values>),
    Function {
        identifier: String,
        args: Vec<Box<Node>>,
        scope: Box<Node>,
    },
    None,
    Break
}

impl Values {
    fn is_none(&self)   -> bool { matches!(self, Values::None) }
    fn is_break(&self)  -> bool { matches!(self, Values::Break) }

    fn name(&self) -> String {
        match self {
            Values::Integer(integer)    => integer.to_string(),
            Values::String(str)         => format!("{:?}", str),
            Values::Boolean(boolean)    => boolean.to_string(),
            Values::Array(values)       => format!("{:?}", values),
            Values::Function {
                identifier,
                ..
            }                           => identifier.to_string(),
            Values::None                => "None".to_string(),
            Values::Break               => "Break".to_string(),
        }
    }
}

#[derive(Debug)]
struct Env {
    vars: HashMap<String, Values>,
    parent: Option<Rc<RefCell<Env>>>,
    cwd: PathBuf
}

impl Env {
    fn new(parent: Option<Rc<RefCell<Env>>>, cwd: PathBuf) -> Self {
        Env {
            vars: HashMap::new(),
            parent,
            cwd
        }
    }

    fn set(&mut self, name: &str, value: Values) {
        self.vars.insert(name.to_string(), value);
    }

    fn update(&mut self, name: &str, value: Values) -> InterpreterResult<Values> {
        if let Some(var) = self.vars.get_mut(name) {
            *var = value.clone();
            return Ok(value.clone())
        }

        if let Some(ref parent) = self.parent {
            return parent.borrow_mut().update(name, value.clone());
        }

        Err(InterpreterError {
            r#type: ErrorTypes::UndefinedVar,
            message: format!("Cannot find var: {:?}", name),
        })
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
            message: format!("Cannot find var: {:?}", name),
        })
    }
}

pub struct Interpreter {
    env: Rc<RefCell<Env>>,
}

impl Interpreter {
    pub fn new(cwd: PathBuf) -> Self {
        Self {
            env: Rc::new(RefCell::new(Env::new(None, cwd))),
        }
    }

    fn handle_fn(&mut self, identifier: &Box<Node>, args: &Vec<Box<Node>>, scope: &Box<Node>) -> InterpreterResult<Values> {
        let identifier = match identifier.deref() {
            Node::Identifier(identifier) => identifier,
            _ => unreachable!(),
        };

        let function = Values::Function {
            identifier: identifier.to_string(),
            args: args.clone(),
            scope: scope.clone(),
        };

        self.env.borrow_mut().set(identifier.as_str(), function);

        Ok(Values::None)
    }

    fn handle_ret(&mut self, node: &Box<Node>) -> InterpreterResult<Values> {
        let value = self.handle_value(node.deref())?;
        Ok(value)
    }

    fn handle_fn_call(&mut self, identifier: &Box<Node>, args: &Vec<Box<Node>>) -> InterpreterResult<Values> {
        let name = match identifier.deref() {
            Node::Identifier(identifier) => identifier,
            _ => unreachable!(),
        };

        let (fn_args, fn_scope) = match self.env.borrow().get(name.as_str()) {
            Ok(Values::Function { args, scope, .. }) => (args, scope),
            _ => {
                return Err(InterpreterError {
                    r#type: ErrorTypes::UndefinedFn,
                    message: format!("Cannot find function: {:?}", name),
                })
            }
        };

        if args.len() != fn_args.len() {
            return Err(InterpreterError {
                r#type: ErrorTypes::TypeError,
                message: format!(
                    "Argument mismatch on function {:?}, Expected {} but found only {}",
                    name,
                    fn_args.len(),
                    args.len()
                ),
            });
        }

        let fn_env = Rc::new(RefCell::new(
            Env::new(
                None,
                self.env.borrow().cwd.clone()
            )
        ));

        for (fn_arg, arg) in fn_args.deref().into_iter().zip(args.deref().into_iter()) {
            if let Node::Identifier(fn_arg) = fn_arg.deref() {
                let val = self.handle_value(arg.deref())?;
                fn_env.borrow_mut().set(fn_arg, val);
            }
        }

        let prev_env = std::mem::replace(&mut self.env, fn_env);

        if let Node::Scope { body } = fn_scope.deref() {
            for scope_node in body {
                let ret_value = self.exec_node(scope_node.deref())?;
                if !ret_value.is_none() {
                    self.env = prev_env;
                    return Ok(ret_value);
                }
            }
        }

        self.env = prev_env;

        Ok(Values::None)
    }

    fn handle_source(&mut self, _file_name: &String, _cwd: &PathBuf, ast: &Vec<Node>) -> InterpreterResult<Values> {
        for node in ast {
            self.exec_node(node)?;
        }

        Ok(Values::None)
    }

    fn handle_scope(&mut self, body: &Vec<Box<Node>>) -> InterpreterResult<Values> {
        let new_env = Rc::new(RefCell::new(
            Env::new(
                Some(self.env.clone()),
                self.env.borrow().cwd.clone()
            )
        ));

        let prev_env = std::mem::replace(&mut self.env, new_env);

        for scope_node in body {
            self.exec_node(scope_node.deref())?;
        }

        self.env = prev_env;

        Ok(Values::None)
    }

    fn handle_math(&mut self, left: &Box<Node>, op: &String, right: &Box<Node>) -> InterpreterResult<Values> {
        let left_value = match left.deref() {
            Node::Literal(literal) => match literal {
                Literals::Int(integer) => integer.clone(),
                _ => return Err(InterpreterError {
                    r#type: ErrorTypes::TypeError,
                    message: format!("Cannot do math on {:?}", literal.name())
                })
            },
            Node::Identifier(identifier) => {
                let variable = self.env.borrow().get(identifier.as_str())?;

                match variable {
                    Values::Integer(integer) => integer,
                    _ => return Err(InterpreterError {
                        r#type: ErrorTypes::TypeError,
                        message: format!("Cannot do math on {:?}", variable)
                    })
                }
            },
            Node::MathExpr { left, op, right } => {
                let nested_result = self.handle_math(left, op, right)?;
                match nested_result {
                    Values::Integer(value) => value,
                    _ => return Err(InterpreterError {
                        r#type: ErrorTypes::TypeError,
                        message: format!("Cannot do math on {:?}", nested_result),
                    }),
                }
            },
            _ => return Err(InterpreterError {
                r#type: ErrorTypes::TypeError,
                message: format!("Cannot do math on {:?}", left)
            })
        };

        let right_value = match right.deref() {
            Node::Literal(literal) => match literal {
                Literals::Int(integer) => integer.clone(),
                _ => return Err(InterpreterError {
                    r#type: ErrorTypes::TypeError,
                    message: format!("Cannot do math on {:?}", literal.name())
                })
            },
            Node::Identifier(identifier) => {
                let variable = self.env.borrow().get(identifier.as_str())?;

                match variable {
                    Values::Integer(integer) => integer,
                    _ => return Err(InterpreterError {
                        r#type: ErrorTypes::TypeError,
                        message: format!("Cannot do math on {:?}", variable)
                    })
                }
            },
            Node::MathExpr { left, op, right } => {
                let nested_result = self.handle_math(left, op, right)?;
                match nested_result {
                    Values::Integer(value) => value,
                    _ => return Err(InterpreterError {
                        r#type: ErrorTypes::TypeError,
                        message: format!("Cannot do math on {:?}", nested_result),
                    }),
                }
            },
            _ => return Err(InterpreterError {
                r#type: ErrorTypes::TypeError,
                message: format!("Cannot do math on {:?}", left)
            })
        };

        match op.as_str() {
            "+" => Ok(Values::Integer(left_value + right_value)),
            "-" => Ok(Values::Integer(left_value - right_value)),
            "*" => Ok(Values::Integer(left_value * right_value)),
            "/" => {
                if right_value == 0 {
                    return Err(InterpreterError {
                        r#type: ErrorTypes::MathError,
                        message: "Division by zero".to_string(),
                    })
                }

                Ok(Values::Integer(left_value / right_value))
            },
            _ => Err(InterpreterError {
                r#type: ErrorTypes::TypeError,
                message: format!("Unknown operator: {}", op),
            }),
        }
    }

    fn handle_random(&mut self, start: &Box<Node>, end: &Box<Node>) -> InterpreterResult<Values> {
        let start = match self.handle_value(start.deref())? {
            Values::Integer(start) => start,
            value => return Err(InterpreterError {
                r#type: ErrorTypes::TypeError,
                message: format!("Cannot generate a random number based on {:?}", value.name())
            })
        };

        let end = match self.handle_value(end.deref())? {
            Values::Integer(end) => end,
            value => return Err(InterpreterError {
                r#type: ErrorTypes::TypeError,
                message: format!("Cannot generate a random number based on {:?}", value.name())
            })
        };

        let mut rng = rand::thread_rng();
        let generated = rng.gen_range(start..end);

        Ok(Values::Integer(generated))
    }

    fn handle_var(&mut self, identifier: &Box<Node>, value: &Box<Node>) -> InterpreterResult<Values> {
        let name = match identifier.deref() {
            Node::Identifier(identifier) => identifier,
            _ => unreachable!(),
        };

        let val = self.handle_value(value.deref())?;
        self.env.borrow_mut().set(name.as_str(), val);

        Ok(Values::None)
    }

    fn handle_array_access(&mut self, identifier: &Box<Node>, index: &Box<Node>) -> InterpreterResult<Values> {
        match identifier.deref() {
            Node::Identifier(name) => {
                let array = match self.env.borrow().get(&name)? {
                    Values::Array(array) => array,
                    _ => return Err(InterpreterError {
                        r#type: ErrorTypes::TypeError,
                        message: format!("Cannot access {:?}. {:?} is not a array", name, name)
                    })
                };

                let index = match self.handle_value(index)? {
                    Values::Integer(index) => index as usize,
                    value => return Err(InterpreterError {
                        r#type: ErrorTypes::TypeError,
                        message: format!("Cannot access {:?} with {:?}", name, value.name())
                    })
                };

                match array.get(index.clone()) {
                    Some(value) => Ok(value.clone()),
                    None => return Err(InterpreterError {
                        r#type: ErrorTypes::IndexOutOfBounds,
                        message: format!("Cannot access {}[{}]", name, index)
                    })
                }
            },
            Node::ArrayAccess { identifier: inner_identifier, index: inner_index } => {
                let inner_value = self.handle_array_access(inner_identifier, inner_index)?;

                let index = match self.handle_value(index)? {
                    Values::Integer(index) => index as usize,
                    value => return Err(InterpreterError {
                        r#type: ErrorTypes::TypeError,
                        message: format!("Cannot access [{:?}] with {:?}", inner_index, value.name())
                    })
                };

                let inner_index = match self.handle_value(inner_index)? {
                    Values::Integer(inner_index) => inner_index as usize,
                    value => return Err(InterpreterError {
                        r#type: ErrorTypes::TypeError,
                        message: format!("Cannot access [{:?}] with {:?}", inner_index, value.name())
                    })
                };

                if let Values::Array(array) = inner_value {
                    match array.get(index.clone()) {
                        Some(value) => Ok(value.clone()),
                        None => return Err(InterpreterError {
                            r#type: ErrorTypes::IndexOutOfBounds,
                            message: format!("Cannot access [{}][{}]", inner_index, index)
                        })
                    }
                } else {
                    Err(InterpreterError {
                        r#type: ErrorTypes::TypeError,
                        message: format!("Expected an array for nested access but found a non-array value")
                    })
                }
            },
            _ => Err(InterpreterError {
                r#type: ErrorTypes::TypeError,
                message: format!("Array access expression is invalid"),
            })
        }
    }

    fn handle_update(&mut self, identifier: &Box<Node>, value: &Box<Node>) -> InterpreterResult<Values> {
        let name = match identifier.deref() {
            Node::Identifier(identifier) => identifier,
            _ => unreachable!(),
        };

        let val = self.handle_value(value.deref())?;

        match self.env.borrow().get(name.as_str()) {
            Ok(variable) => {
                if discriminant(&val) != discriminant(&variable) {
                    return Err(InterpreterError {
                        r#type: ErrorTypes::TypeError,
                        message: format!(
                            "Cannot update variable with different type: {:?} -> {:?}",
                            val.name(),
                            variable.name()
                        ),
                    });
                }
            }
            Err(err) => return Err(err),
        }

        self.env.borrow_mut().update(name.as_str(), val)?;

        Ok(Values::None)
    }

    fn handle_log(&mut self, log_type: &str, args: &Vec<Box<Node>>) -> InterpreterResult<Values> {
        let mut output = String::new();

        for arg in args {
            let value = self.handle_value(arg.deref())?;

            match value {
                Values::Integer(integer)    => output.push_str(integer.to_string().as_str()),
                Values::String(str)         => output.push_str(str.as_str()),
                Values::Boolean(boolean)    => output.push_str(boolean.to_string().as_str()),
                Values::Array(values)       => output.push_str(
                    (values.iter()
                        .map(|value| value.name())
                        .collect::<Vec<String>>())
                        .join(" ")
                        .as_str()
                ),
                _ => {
                    return Err(InterpreterError {
                        r#type: ErrorTypes::UnknownError,
                        message: format!("Something went wrong while handling log args"),
                    })
                }
            }
        }

        match log_type {
            "log"   => print!("{output}"),
            "logl"  => println!("{output}"),
            _       => (),
        }

        Ok(Values::None)
    }

    fn handle_check(&mut self, condition: &Box<Node>, scope: &Box<Node>) -> InterpreterResult<Values> {
        let new_env = Rc::new(RefCell::new(
            Env::new(
                Some(self.env.clone()),
                self.env.borrow().cwd.clone()
            )
        ));

        let prev_env = std::mem::replace(&mut self.env, new_env);

        if let Values::Boolean(condition) = self.handle_condition(condition)? {
            if condition {
                if let Node::Scope { body } = scope.deref() {
                    for scope_node in body {
                        if let Node::Break = scope_node.deref() {
                            return Ok(Values::Break)
                        }

                        let ret_value = self.exec_node(scope_node.deref())?;
                        if !ret_value.is_none() {
                            return Ok(ret_value);
                        }
                    }
                }
            }
        }

        self.env = prev_env;

        Ok(Values::None)
    }

    fn handle_while(&mut self, condition: &Box<Node>, scope: &Box<Node>) -> InterpreterResult<Values> {
        let new_env = Rc::new(RefCell::new(
            Env::new(
                Some(self.env.clone()),
                self.env.borrow().cwd.clone()
            )
        ));

        let prev_env = std::mem::replace(&mut self.env, new_env);

        while let Values::Boolean(condition) = self.handle_condition(condition)? {
            if !condition {
                break;
            }

            if let Node::Scope { body } = scope.deref() {
                for scope_node in body {
                    if let Node::Break = scope_node.deref() {
                        return Ok(Values::None);
                    }

                    let ret_value = self.exec_node(scope_node.deref())?;
                    if ret_value.is_break() {
                        return Ok(Values::None);
                    }

                    if !ret_value.is_none() {
                        return Ok(ret_value);
                    }
                }
            }
        }

        self.env = prev_env;

        Ok(Values::None)
    }

    fn handle_array(&mut self, values: &Vec<Literals>) -> InterpreterResult<Values> {
        let mut parsed_values: Vec<Values> = vec![];

        for value in values {
            let value = match value {
                Literals::Int(integer)      => Values::Integer(integer.clone()),
                Literals::String(str)       => Values::String(str.clone()),
                Literals::Boolean(boolean)  => Values::Boolean(boolean.clone()),
                Literals::Array(values)     => self.handle_array(values)?
            };

            parsed_values.push(value);
        }

        Ok(Values::Array(parsed_values))
    }

    fn handle_value(&mut self, node: &Node) -> InterpreterResult<Values> {
        match node {
            Node::Literal(Literals::Int(integer))       => Ok(Values::Integer(integer.clone())),
            Node::Literal(Literals::String(str))        => Ok(Values::String(str.clone())),
            Node::Literal(Literals::Boolean(boolean))   => Ok(Values::Boolean(boolean.clone())),
            Node::Literal(Literals::Array(values))      => self.handle_array(values),
            Node::ArrayAccess { identifier, index }     => self.handle_array_access(identifier, index),
            Node::Identifier(identifier)                => self.env.borrow().get(identifier.as_str()),
            Node::FunctionCall { identifier, args }     => self.handle_fn_call(identifier, args),
            Node::MathExpr { left, op, right }          => self.handle_math(left, op, right),
            Node::Random { start, end }                 => self.handle_random(start, end),
            _ => Err(InterpreterError {
                r#type: ErrorTypes::UnknownError,
                message: format!("Something went wrong while handling value"),
            }),
        }
    }

    fn handle_condition(&mut self, condition: &Box<Node>) -> InterpreterResult<Values> {
        match condition.deref() {
            Node::Condition { left, condition, right } => {
                let left_value = self.handle_value(left.deref())?;
                let right_value = self.handle_value(right.deref())?;

                match (left_value, right_value) {
                    (Values::Integer(left_int), Values::Integer(right_int))         => Ok(Values::Boolean(compare!(left_int, condition, right_int))),
                    (Values::String(left_str), Values::String(right_str))           => Ok(Values::Boolean(compare!(left_str, condition, right_str))),
                    (Values::Boolean(left_boolean), Values::Boolean(right_boolean)) => Ok(Values::Boolean(compare!(left_boolean, condition, right_boolean))),
                    _ => {
                        return Err(InterpreterError {
                            r#type: ErrorTypes::TypeError,
                            message: format!("Cannot compare {:?} to {:?}", left, right),
                        })
                    }
                }
            }
            Node::Literal(literal) => match literal {
                Literals::Int(integer)      => Ok(Values::Boolean(*integer > 0)),
                Literals::String(str)       => Ok(Values::Boolean(str.len() > 0)),
                Literals::Boolean(boolean)  => Ok(Values::Boolean(*boolean)),
                Literals::Array(values)     => Ok(Values::Boolean(values.len() > 0)),
            },
            _ => {
                return Err(InterpreterError {
                    r#type: ErrorTypes::UnknownError,
                    message: format!("Something went wrong in handle_while"),
                })
            }
        }
    }

    fn exec_node(&mut self, node: &Node) -> InterpreterResult<Values> {
        match node {
            Node::Function { identifier, args, scope }  => self.handle_fn(identifier, args, scope),
            Node::FunctionCall { identifier, args }     => self.handle_fn_call(identifier, args),
            Node::Return(value)                         => self.handle_ret(value),
            Node::Source { file_name, cwd, ast }        => self.handle_source(file_name, cwd, ast),
            Node::Scope { body }                        => self.handle_scope(body),
            Node::MathExpr { left, op, right }          => self.handle_math(left, op, right),
            Node::Random { start, end }                 => self.handle_random(start, end),
            Node::Var { identifier, value }             => self.handle_var(identifier, value),
            Node::Update { identifier, value }          => self.handle_update(identifier, value),
            Node::Check { condition, scope }            => self.handle_check(condition, scope),
            Node::While { condition, scope }            => self.handle_while(condition, scope),
            Node::Log { r#type, args }                  => self.handle_log(r#type.as_str(), args),
            _                                           => Ok(Values::None),
        }
    }

    pub fn run(&mut self, ast: &Vec<Node>) -> InterpreterResult<()> {
        for node in ast {
            self.exec_node(node)?;
        }

        Ok(())
    }
}
