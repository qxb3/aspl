use crate::{lexer::TokenTypes, parser::{ExprConditional, ExprNodeTypes, LiteralTypes, Node, NodeTypes, NodeVariable}};
use std::iter::Peekable;
use inline_colorization::*;

macro_rules! compare {
    ($self:ident, $left_value:expr, $conditional_type:expr, $right_value:expr) => {
        match $conditional_type {
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

    fn handle_log(&self, nodes: Peekable<std::slice::Iter<Node>>, args: &Vec<ExprNodeTypes>, log_type: &str) -> Result<(), String> {
        let mut output = String::new();

        for arg in args {
            match arg {
                ExprNodeTypes::Literal(LiteralTypes::String(value)) => { output.push_str(value.as_str()); },
                ExprNodeTypes::Literal(LiteralTypes::Int(value)) => { output.push_str(value.to_string().as_str()); },
                ExprNodeTypes::Literal(LiteralTypes::Boolean(value)) => { output.push_str(value.to_string().as_str()); },
                ExprNodeTypes::Identifier(name) => {
                    let result = self.find_variable(nodes.clone(), name.to_string());

                    match &result {
                        Ok(variable) => match &variable.value {
                            ExprNodeTypes::Literal(LiteralTypes::String(value)) => { output.push_str(value.as_str()); },
                            ExprNodeTypes::Literal(LiteralTypes::Int(value)) => { output.push_str(value.to_string().as_str()); },
                            ExprNodeTypes::Literal(LiteralTypes::Boolean(value)) => { output.push_str(value.to_string().as_str()); },
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

    fn handle_check(&self, ast: &Vec<Node>, conditional_node: &ExprConditional, mut childrens: Peekable<std::slice::Iter<Node>>) -> Result<(), String> {
        match self.handle_conditional(ast, &conditional_node.left, &conditional_node.condition_type, &conditional_node.right) {
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

    fn handle_while(&self, ast: &Vec<Node>, conditional_node: &ExprConditional, childrens: &Vec<Node>) -> Result<(), String> {
        loop {
            let mut childrens = childrens.iter().peekable();

            match self.handle_conditional(ast, &conditional_node.left, &conditional_node.condition_type, &conditional_node.right) {
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

    fn handle_conditional(&self, ast: &Vec<Node>, left: &ExprNodeTypes, conditional_type: &TokenTypes, right: &ExprNodeTypes) -> Result<bool, String> {
        match (left, right) {
            (ExprNodeTypes::Literal(LiteralTypes::Int(left_value)), ExprNodeTypes::Literal(LiteralTypes::Int(right_value))) => {
                compare!(self, left_value, conditional_type, right_value)
            },
            (ExprNodeTypes::Identifier(left_name), ExprNodeTypes::Literal(LiteralTypes::Int(right_value))) => {
                let left_result = self.find_variable(ast.iter().peekable(), left_name.to_string());

                match &left_result {
                    Ok(left_variable) => match &left_variable.value {
                        ExprNodeTypes::Literal(LiteralTypes::Int(left_value)) => {
                            compare!(self, left_value, conditional_type, right_value)
                        },
                        ExprNodeTypes::Literal(LiteralTypes::String(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to int: {}.", left_value, right_value)),
                        ExprNodeTypes::Literal(LiteralTypes::Boolean(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to int: {}.", left_value, right_value)),
                        _ => unreachable!()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", left_name))
                }
            },
            (ExprNodeTypes::Literal(LiteralTypes::Int(left_value)), ExprNodeTypes::Identifier(right_name)) => {
                let right_result = self.find_variable(ast.iter().peekable(), right_name.to_string());

                match &right_result {
                    Ok(right_variable) => match &right_variable.value {
                        ExprNodeTypes::Literal(LiteralTypes::Int(right_value)) => {
                            compare!(self, left_value, conditional_type, right_value)
                        },
                        ExprNodeTypes::Literal(LiteralTypes::String(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to int: {}.", left_value, right_name)),
                        ExprNodeTypes::Literal(LiteralTypes::Boolean(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to int: {}.", left_value, right_name)),
                        _ => unreachable!()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", right_name))
                }
            },
            (ExprNodeTypes::Literal(LiteralTypes::String(left_value)), ExprNodeTypes::Literal(LiteralTypes::String(right_value))) => {
                compare!(self, left_value, conditional_type, right_value)
            },
            (ExprNodeTypes::Identifier(left_name), ExprNodeTypes::Literal(LiteralTypes::String(right_value))) => {
                let left_result = self.find_variable(ast.iter().peekable(), left_name.to_string());

                match &left_result {
                    Ok(left_variable) => match &left_variable.value {
                        ExprNodeTypes::Literal(LiteralTypes::String(left_value)) => {
                            compare!(self, left_value, conditional_type, right_value)
                        },
                        ExprNodeTypes::Literal(LiteralTypes::Int(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare int: {} to string: \"{}\".", left_value, right_value)),
                        ExprNodeTypes::Literal(LiteralTypes::Boolean(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to string: \"{}\".", left_value, right_value)),
                        _ => unreachable!()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", left_name))
                }
            },
            (ExprNodeTypes::Literal(LiteralTypes::String(left_value)), ExprNodeTypes::Identifier(right_name)) => {
                let right_result = self.find_variable(ast.iter().peekable(), right_name.to_string());

                match &right_result {
                    Ok(right_variable) => match &right_variable.value {
                        ExprNodeTypes::Literal(LiteralTypes::String(right_value)) => {
                            compare!(self, left_value, conditional_type, right_value)
                        },
                        ExprNodeTypes::Literal(LiteralTypes::Int(right_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to int: {}.", left_value, right_value)),
                        ExprNodeTypes::Literal(LiteralTypes::Boolean(right_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to boolean: {}.", left_value, right_value)),
                        _ => unreachable!()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", right_name))
                }
            },
            (ExprNodeTypes::Literal(LiteralTypes::Boolean(left_value)), ExprNodeTypes::Literal(LiteralTypes::Boolean(right_value))) => {
                compare!(self, left_value, conditional_type, right_value)
            },
            (ExprNodeTypes::Identifier(left_name), ExprNodeTypes::Literal(LiteralTypes::Boolean(right_value))) => {
                let left_result = self.find_variable(ast.iter().peekable(), left_name.to_string());

                match &left_result {
                    Ok(left_variable) => match &left_variable.value {
                        ExprNodeTypes::Literal(LiteralTypes::Boolean(left_value)) => {
                            compare!(self, left_value, conditional_type, right_value)
                        },
                        ExprNodeTypes::Literal(LiteralTypes::Int(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare int: {} to boolean: {}.", left_value, right_value)),
                        ExprNodeTypes::Literal(LiteralTypes::String(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to boolean: {}.", left_value, right_value)),
                        _ => unreachable!()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", left_name))
                }
            },
            (ExprNodeTypes::Literal(LiteralTypes::Boolean(left_value)), ExprNodeTypes::Identifier(right_name)) => {
                let right_result = self.find_variable(ast.iter().peekable(), right_name.to_string());

                match &right_result {
                    Ok(right_variable) => match &right_variable.value {
                        ExprNodeTypes::Literal(LiteralTypes::Boolean(right_value)) => {
                            compare!(self, left_value, conditional_type, right_value)
                        },
                        ExprNodeTypes::Literal(LiteralTypes::Int(right_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to int: {}.", left_value, right_value)),
                        ExprNodeTypes::Literal(LiteralTypes::String(right_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to string: \"{}\".", left_value, right_value)),
                        _ => unreachable!()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", right_name))
                }
            },
            (ExprNodeTypes::Identifier(left_name), ExprNodeTypes::Identifier(right_name)) => {
                let left_result = self.find_variable(ast.iter().peekable(), left_name.to_string());
                let right_result = self.find_variable(ast.iter().peekable(), right_name.to_string());

                match (&left_result, &right_result) {
                    (Ok(left_variable), Ok(right_variable)) => {
                        match (&left_variable.value, &right_variable.value) {
                            (ExprNodeTypes::Literal(LiteralTypes::Int(left_value)), ExprNodeTypes::Literal(LiteralTypes::Int(right_value))) => {
                                compare!(self, left_value, conditional_type, right_value)
                            },
                            (ExprNodeTypes::Literal(LiteralTypes::String(left_value)), ExprNodeTypes::Literal(LiteralTypes::Int(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to int: {}.", left_value, right_value)),
                            (ExprNodeTypes::Literal(LiteralTypes::Int(left_value)), ExprNodeTypes::Literal(LiteralTypes::String(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare int: {} to string: \"{}\".", left_value, right_value)),
                            (ExprNodeTypes::Literal(LiteralTypes::Boolean(left_value)), ExprNodeTypes::Literal(LiteralTypes::Int(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to int: {}.", left_value, right_value)),
                            (ExprNodeTypes::Literal(LiteralTypes::Int(left_value)), ExprNodeTypes::Literal(LiteralTypes::Boolean(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare int: {} to boolean: {}.", left_value, right_value)),
                            _ => unreachable!()
                        }
                    },
                    (Err(_), _) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", left_name)),
                    (_, Err(_)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", right_name))
                }
            },
            (ExprNodeTypes::Literal(LiteralTypes::Int(left_value)), ExprNodeTypes::Literal(LiteralTypes::String(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare int: {} to string \"{}\".", left_value, right_value)),
            (ExprNodeTypes::Literal(LiteralTypes::Int(left_value)), ExprNodeTypes::Literal(LiteralTypes::Boolean(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare int: {} to boolean {}.", left_value, right_value)),
            (ExprNodeTypes::Literal(LiteralTypes::String(left_value)), ExprNodeTypes::Literal(LiteralTypes::Int(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to int {}.", left_value, right_value)),
            (ExprNodeTypes::Literal(LiteralTypes::String(left_value)), ExprNodeTypes::Literal(LiteralTypes::Boolean(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to boolean {}.", left_value, right_value)),
            (ExprNodeTypes::Literal(LiteralTypes::Boolean(left_value)), ExprNodeTypes::Literal(LiteralTypes::Int(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to int {}.", left_value, right_value)),
            (ExprNodeTypes::Literal(LiteralTypes::Boolean(left_value)), ExprNodeTypes::Literal(LiteralTypes::String(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to string \"{}\".", left_value, right_value))
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
            NodeTypes::Check(conditional_node, children) => {
                if let Err(err) = self.handle_check(ast, conditional_node, children.iter().peekable()) {
                    return Err(err);
                }
            },
            NodeTypes::While(conditional_node, children) => {
                if let Err(err) = self.handle_while(ast, conditional_node, children) {
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
