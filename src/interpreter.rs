use crate::{lexer::TokenTypes, parser::{ExprCondition, Expr, Literals, Node, NodeTypes, NodeVariable}};
use std::iter::Peekable;
use inline_colorization::*;

macro_rules! compare {
    ($self:ident, $left_value:expr, $condition_type:expr, $right_value:expr) => {
        match $condition_type {
            TokenTypes::EqEq => Ok($self.compare($left_value, $right_value, |a, b| a == b)),
            TokenTypes::NotEq => Ok($self.compare($left_value, $right_value, |a, b| a != b)),
            TokenTypes::GThan => Ok($self.compare($left_value, $right_value, |a, b| a > b)),
            TokenTypes::GThanEq => Ok($self.compare($left_value, $right_value, |a, b| a >= b)),
            TokenTypes::LThan => Ok($self.compare($left_value, $right_value, |a, b| a < b)),
            TokenTypes::LThanEq => Ok($self.compare($left_value, $right_value, |a, b| a <= b)),
            _ => unreachable!()
        }
    };
}

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    fn handle_log(&self, nodes: Peekable<std::slice::Iter<Node>>, args: &Vec<Expr>, log_type: &str) -> Result<(), String> {
        let mut output = String::new();

        for arg in args {
            match arg {
                Expr::Literal(Literals::String(value)) => { output.push_str(value.as_str()); },
                Expr::Literal(Literals::Int(value)) => { output.push_str(value.to_string().as_str()); },
                Expr::Literal(Literals::Boolean(value)) => { output.push_str(value.to_string().as_str()); },
                Expr::Identifier(name) => {
                    let result = self.find_variable(nodes.clone(), name.to_string());

                    match &result {
                        Ok(variable) => match &variable.value {
                            Expr::Literal(Literals::String(value)) => { output.push_str(value.as_str()); },
                            Expr::Literal(Literals::Int(value)) => { output.push_str(value.to_string().as_str()); },
                            Expr::Literal(Literals::Boolean(value)) => { output.push_str(value.to_string().as_str()); },
                            _ => ()
                        },
                        Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", name))
                    }
                }
            }
        }

        match log_type {
            "log" => { print!("{output}"); },
            "logl" => { println!("{output}"); }
            _ => ()
        }

        Ok(())
    }

    fn handle_check(&self, ast: &Vec<Node>, condition_node: &ExprCondition, mut childrens: Peekable<std::slice::Iter<Node>>) -> Result<(), String> {
        match self.handle_condition(ast, &condition_node.left, &condition_node.condition_type, &condition_node.right) {
            Ok(result) => {
                if result {
                    while let Some(curr_node) = childrens.next() {
                        if let Err(err) = self.execute_node(ast, curr_node) {
                            return Err(err);
                        }
                    }
                }
            },
            Err(err) => return Err(err)
        }

        Ok(())
    }

    fn handle_while(&self, ast: &Vec<Node>, condition_node: &ExprCondition, childrens: &Vec<Node>) -> Result<(), String> {
        loop {
            let mut childrens = childrens.iter().peekable();

            match self.handle_condition(ast, &condition_node.left, &condition_node.condition_type, &condition_node.right) {
                Ok(result) => {
                    if !result { break; }

                    while let Some(curr_node) = childrens.next() {
                        if let Err(err) = self.execute_node(ast, curr_node) {
                            return Err(err);
                        }
                    }
                },
                Err(err) => return Err(err)
            }
        }

        Ok(())
    }

    fn handle_condition(&self, ast: &Vec<Node>, left: &Expr, condition_type: &TokenTypes, right: &Expr) -> Result<bool, String> {
        match (left, right) {
            (Expr::Literal(Literals::Int(left_value)), Expr::Literal(Literals::Int(right_value))) => {
                compare!(self, left_value, condition_type, right_value)
            },
            (Expr::Identifier(left_name), Expr::Literal(Literals::Int(right_value))) => {
                let left_result = self.find_variable(ast.iter().peekable(), left_name.to_string());

                match &left_result {
                    Ok(left_variable) => match &left_variable.value {
                        Expr::Literal(Literals::Int(left_value)) => {
                            compare!(self, left_value, condition_type, right_value)
                        },
                        Expr::Literal(Literals::String(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to int: {}.", left_value, right_value)),
                        Expr::Literal(Literals::Boolean(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to int: {}.", left_value, right_value)),
                        _ => unreachable!()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", left_name))
                }
            },
            (Expr::Literal(Literals::Int(left_value)), Expr::Identifier(right_name)) => {
                let right_result = self.find_variable(ast.iter().peekable(), right_name.to_string());

                match &right_result {
                    Ok(right_variable) => match &right_variable.value {
                        Expr::Literal(Literals::Int(right_value)) => {
                            compare!(self, left_value, condition_type, right_value)
                        },
                        Expr::Literal(Literals::String(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to int: {}.", left_value, right_name)),
                        Expr::Literal(Literals::Boolean(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to int: {}.", left_value, right_name)),
                        _ => unreachable!()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", right_name))
                }
            },
            (Expr::Literal(Literals::String(left_value)), Expr::Literal(Literals::String(right_value))) => {
                compare!(self, left_value, condition_type, right_value)
            },
            (Expr::Identifier(left_name), Expr::Literal(Literals::String(right_value))) => {
                let left_result = self.find_variable(ast.iter().peekable(), left_name.to_string());

                match &left_result {
                    Ok(left_variable) => match &left_variable.value {
                        Expr::Literal(Literals::String(left_value)) => {
                            compare!(self, left_value, condition_type, right_value)
                        },
                        Expr::Literal(Literals::Int(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare int: {} to string: \"{}\".", left_value, right_value)),
                        Expr::Literal(Literals::Boolean(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to string: \"{}\".", left_value, right_value)),
                        _ => unreachable!()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", left_name))
                }
            },
            (Expr::Literal(Literals::String(left_value)), Expr::Identifier(right_name)) => {
                let right_result = self.find_variable(ast.iter().peekable(), right_name.to_string());

                match &right_result {
                    Ok(right_variable) => match &right_variable.value {
                        Expr::Literal(Literals::String(right_value)) => {
                            compare!(self, left_value, condition_type, right_value)
                        },
                        Expr::Literal(Literals::Int(right_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to int: {}.", left_value, right_value)),
                        Expr::Literal(Literals::Boolean(right_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to boolean: {}.", left_value, right_value)),
                        _ => unreachable!()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", right_name))
                }
            },
            (Expr::Literal(Literals::Boolean(left_value)), Expr::Literal(Literals::Boolean(right_value))) => {
                compare!(self, left_value, condition_type, right_value)
            },
            (Expr::Identifier(left_name), Expr::Literal(Literals::Boolean(right_value))) => {
                let left_result = self.find_variable(ast.iter().peekable(), left_name.to_string());

                match &left_result {
                    Ok(left_variable) => match &left_variable.value {
                        Expr::Literal(Literals::Boolean(left_value)) => {
                            compare!(self, left_value, condition_type, right_value)
                        },
                        Expr::Literal(Literals::Int(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare int: {} to boolean: {}.", left_value, right_value)),
                        Expr::Literal(Literals::String(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to boolean: {}.", left_value, right_value)),
                        _ => unreachable!()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", left_name))
                }
            },
            (Expr::Literal(Literals::Boolean(left_value)), Expr::Identifier(right_name)) => {
                let right_result = self.find_variable(ast.iter().peekable(), right_name.to_string());

                match &right_result {
                    Ok(right_variable) => match &right_variable.value {
                        Expr::Literal(Literals::Boolean(right_value)) => {
                            compare!(self, left_value, condition_type, right_value)
                        },
                        Expr::Literal(Literals::Int(right_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to int: {}.", left_value, right_value)),
                        Expr::Literal(Literals::String(right_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to string: \"{}\".", left_value, right_value)),
                        _ => unreachable!()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", right_name))
                }
            },
            (Expr::Identifier(left_name), Expr::Identifier(right_name)) => {
                let left_result = self.find_variable(ast.iter().peekable(), left_name.to_string());
                let right_result = self.find_variable(ast.iter().peekable(), right_name.to_string());

                match (&left_result, &right_result) {
                    (Ok(left_variable), Ok(right_variable)) => {
                        match (&left_variable.value, &right_variable.value) {
                            (Expr::Literal(Literals::Int(left_value)), Expr::Literal(Literals::Int(right_value))) => {
                                compare!(self, left_value, condition_type, right_value)
                            },
                            (Expr::Literal(Literals::String(left_value)), Expr::Literal(Literals::Int(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to int: {}.", left_value, right_value)),
                            (Expr::Literal(Literals::Int(left_value)), Expr::Literal(Literals::String(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare int: {} to string: \"{}\".", left_value, right_value)),
                            (Expr::Literal(Literals::Boolean(left_value)), Expr::Literal(Literals::Int(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to int: {}.", left_value, right_value)),
                            (Expr::Literal(Literals::Int(left_value)), Expr::Literal(Literals::Boolean(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare int: {} to boolean: {}.", left_value, right_value)),
                            _ => unreachable!()
                        }
                    },
                    (Err(_), _) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", left_name)),
                    (_, Err(_)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", right_name))
                }
            },
            (Expr::Literal(Literals::Int(left_value)), Expr::Literal(Literals::String(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare int: {} to string \"{}\".", left_value, right_value)),
            (Expr::Literal(Literals::Int(left_value)), Expr::Literal(Literals::Boolean(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare int: {} to boolean {}.", left_value, right_value)),
            (Expr::Literal(Literals::String(left_value)), Expr::Literal(Literals::Int(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to int {}.", left_value, right_value)),
            (Expr::Literal(Literals::String(left_value)), Expr::Literal(Literals::Boolean(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to boolean {}.", left_value, right_value)),
            (Expr::Literal(Literals::Boolean(left_value)), Expr::Literal(Literals::Int(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to int {}.", left_value, right_value)),
            (Expr::Literal(Literals::Boolean(left_value)), Expr::Literal(Literals::String(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to string \"{}\".", left_value, right_value))
        }
    }

    fn execute_node(&self, ast: &Vec<Node>, node: &Node) -> Result<(), String> {
        match &node.r#type {
            NodeTypes::Log(args) => {
                if let Err(err) = self.handle_log(ast.iter().peekable(), args, "log") {
                    return Err(err);
                }
            },
            NodeTypes::Logl(args) => {
                if let Err(err) = self.handle_log(ast.iter().peekable(), args, "logl") {
                    return Err(err);
                }
            },
            NodeTypes::Check(condition_node, children) => {
                if let Err(err) = self.handle_check(ast, condition_node, children.iter().peekable()) {
                    return Err(err);
                }
            },
            NodeTypes::While(condition_node, children) => {
                if let Err(err) = self.handle_while(ast, condition_node, children) {
                    return Err(err);
                }
            },
            _ => {}
        }

        Ok(())
    }

    pub fn run(&self, ast: Vec<Node>) -> Result<(), String> {
        let mut nodes = ast.iter().peekable();

        while let Some(node) = nodes.next() {
            if let Err(err) = self.execute_node(&ast, node) {
                return Err(err);
            }
        }

        Ok(())
    }

    fn compare<T, F>(&self, a: T, b: T, f: F) -> bool
    where
        F: Fn(T, T) -> bool
    {
        f(a, b)
    }

    fn find_variable(&self, mut nodes: Peekable<std::slice::Iter<Node>>, name: String) -> Result<NodeVariable, ()> {
        while let Some(node) = nodes.next() {
            match &node.r#type {
                NodeTypes::Variable(variable) => {
                    if variable.name == name {
                        return Ok(variable.clone());
                    }
                },
                NodeTypes::Check(_, children) => {
                    if let Ok(variable) = self.find_variable(children.iter().peekable(), name.clone()) {
                        return Ok(variable);
                    }
                },
                _ => {}
            }
        }

        Err(())
    }
}
