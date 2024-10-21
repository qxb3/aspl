use crate::{lexer::TokenTypes, parser::{ComparisonNode, ExprNodeTypes, LiteralTypes, Node, NodeTypes, VariableNode}};
use std::iter::Peekable;
use inline_colorization::*;

macro_rules! compare {
    ($self:ident, $childrens:ident, $ast:ident, $left:expr, $op:tt, $right:expr) => {
        match ($left, $right) {
            (ExprNodeTypes::Literal(LiteralTypes::Int(left_value)), ExprNodeTypes::Literal(LiteralTypes::Int(right_value))) => {
                if left_value $op right_value {
                    while let Some(curr_node) = $childrens.next() {
                        if let Err(err) = $self.execute_node($ast, curr_node) {
                            return Err(err);
                        }
                    }
                }
            },
            (ExprNodeTypes::Identifier(left_name), ExprNodeTypes::Literal(LiteralTypes::Int(right_value))) => {
                let left_result = $self.find_variable($ast.iter().peekable(), left_name.to_string());

                match &left_result {
                    Ok(left_variable) => match &left_variable.value {
                        ExprNodeTypes::Literal(LiteralTypes::Int(left_value)) => {
                            if left_value $op right_value {
                                while let Some(curr_node) = $childrens.next() {
                                    if let Err(err) = $self.execute_node($ast, curr_node) {
                                        return Err(err);
                                    }
                                }
                            }
                        },
                        ExprNodeTypes::Literal(LiteralTypes::String(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to int: {}.", left_value, right_value)),
                        ExprNodeTypes::Literal(LiteralTypes::Boolean(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to int: {}.", left_value, right_value)),
                        _ => ()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", left_name))
                }
            },
            (ExprNodeTypes::Literal(LiteralTypes::Int(left_value)), ExprNodeTypes::Identifier(right_name)) => {
                let right_result = $self.find_variable($ast.iter().peekable(), right_name.to_string());

                match &right_result {
                    Ok(right_variable) => match &right_variable.value {
                        ExprNodeTypes::Literal(LiteralTypes::Int(right_value)) => {
                            if left_value $op right_value {
                                while let Some(curr_node) = $childrens.next() {
                                    if let Err(err) = $self.execute_node($ast, curr_node) {
                                        return Err(err);
                                    }
                                }
                            }
                        },
                        ExprNodeTypes::Literal(LiteralTypes::String(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to int: {}.", left_value, right_name)),
                        ExprNodeTypes::Literal(LiteralTypes::Boolean(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to int: {}.", left_value, right_name)),
                        _ => ()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", right_name))
                }
            },
            (ExprNodeTypes::Literal(LiteralTypes::String(left_value)), ExprNodeTypes::Literal(LiteralTypes::String(right_value))) => {
                if left_value $op right_value {
                    while let Some(curr_node) = $childrens.next() {
                        if let Err(err) = $self.execute_node($ast, curr_node) {
                            return Err(err);
                        }
                    }
                }
            },
            (ExprNodeTypes::Identifier(left_name), ExprNodeTypes::Literal(LiteralTypes::String(right_value))) => {
                let left_result = $self.find_variable($ast.iter().peekable(), left_name.to_string());

                match &left_result {
                    Ok(left_variable) => match &left_variable.value {
                        ExprNodeTypes::Literal(LiteralTypes::String(left_value)) => {
                            if left_value $op right_value {
                                while let Some(curr_node) = $childrens.next() {
                                    if let Err(err) = $self.execute_node($ast, curr_node) {
                                        return Err(err);
                                    }
                                }
                            }
                        },
                        ExprNodeTypes::Literal(LiteralTypes::Int(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare int: {} to string: \"{}\".", left_value, right_value)),
                        ExprNodeTypes::Literal(LiteralTypes::Boolean(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to string: \"{}\".", left_value, right_value)),
                        _ => ()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", left_name))
                }
            },
            (ExprNodeTypes::Literal(LiteralTypes::String(left_value)), ExprNodeTypes::Identifier(right_name)) => {
                let right_result = $self.find_variable($ast.iter().peekable(), right_name.to_string());

                match &right_result {
                    Ok(right_variable) => match &right_variable.value {
                        ExprNodeTypes::Literal(LiteralTypes::String(right_value)) => {
                            if left_value $op right_value {
                                while let Some(curr_node) = $childrens.next() {
                                    if let Err(err) = $self.execute_node($ast, curr_node) {
                                        return Err(err);
                                    }
                                }
                            }
                        },
                        ExprNodeTypes::Literal(LiteralTypes::Int(right_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to int: {}.", left_value, right_value)),
                        ExprNodeTypes::Literal(LiteralTypes::Boolean(right_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to boolean: {}.", left_value, right_value)),
                        _ => ()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", right_name))
                }
            },
            (ExprNodeTypes::Literal(LiteralTypes::Boolean(left_value)), ExprNodeTypes::Literal(LiteralTypes::Boolean(right_value))) => {
                if left_value $op right_value {
                    while let Some(curr_node) = $childrens.next() {
                        if let Err(err) = $self.execute_node($ast, curr_node) {
                            return Err(err);
                        }
                    }
                }
            },
            (ExprNodeTypes::Identifier(left_name), ExprNodeTypes::Literal(LiteralTypes::Boolean(right_value))) => {
                let left_result = $self.find_variable($ast.iter().peekable(), left_name.to_string());

                match &left_result {
                    Ok(left_variable) => match &left_variable.value {
                        ExprNodeTypes::Literal(LiteralTypes::Boolean(left_value)) => {
                            if left_value $op right_value {
                                while let Some(curr_node) = $childrens.next() {
                                    if let Err(err) = $self.execute_node($ast, curr_node) {
                                        return Err(err);
                                    }
                                }
                            }
                        },
                        ExprNodeTypes::Literal(LiteralTypes::Int(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare int: {} to boolean: {}.", left_value, right_value)),
                        ExprNodeTypes::Literal(LiteralTypes::String(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to boolean: {}.", left_value, right_value)),
                        _ => ()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", left_name))
                }
            },
            (ExprNodeTypes::Literal(LiteralTypes::Boolean(left_value)), ExprNodeTypes::Identifier(right_name)) => {
                let right_result = $self.find_variable($ast.iter().peekable(), right_name.to_string());

                match &right_result {
                    Ok(right_variable) => match &right_variable.value {
                        ExprNodeTypes::Literal(LiteralTypes::Boolean(right_value)) => {
                            if left_value $op right_value {
                                while let Some(curr_node) = $childrens.next() {
                                    if let Err(err) = $self.execute_node($ast, curr_node) {
                                        return Err(err);
                                    }
                                }
                            }
                        },
                        ExprNodeTypes::Literal(LiteralTypes::Int(right_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to int: {}.", left_value, right_value)),
                        ExprNodeTypes::Literal(LiteralTypes::String(right_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to string: \"{}\".", left_value, right_value)),
                        _ => ()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", right_name))
                }
            },
            (ExprNodeTypes::Identifier(left_name), ExprNodeTypes::Identifier(right_name)) => {
                let left_result = $self.find_variable($ast.iter().peekable(), left_name.to_string());
                let right_result = $self.find_variable($ast.iter().peekable(), right_name.to_string());

                match (&left_result, &right_result) {
                    (Ok(left_variable), Ok(right_variable)) => {
                        match (&left_variable.value, &right_variable.value) {
                            (ExprNodeTypes::Literal(LiteralTypes::Int(left_value)), ExprNodeTypes::Literal(LiteralTypes::Int(right_value))) => {
                                if left_value $op right_value {
                                    while let Some(curr_node) = $childrens.next() {
                                        if let Err(err) = $self.execute_node($ast, curr_node) {
                                            return Err(err);
                                        }
                                    }
                                }
                            },
                            (ExprNodeTypes::Literal(LiteralTypes::String(left_value)), ExprNodeTypes::Literal(LiteralTypes::Int(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to int: {}.", left_value, right_value)),
                            (ExprNodeTypes::Literal(LiteralTypes::Int(left_value)), ExprNodeTypes::Literal(LiteralTypes::String(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare int: {} to string: \"{}\".", left_value, right_value)),
                            (ExprNodeTypes::Literal(LiteralTypes::Boolean(left_value)), ExprNodeTypes::Literal(LiteralTypes::Int(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to int: {}.", left_value, right_value)),
                            (ExprNodeTypes::Literal(LiteralTypes::Int(left_value)), ExprNodeTypes::Literal(LiteralTypes::Boolean(right_value))) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare int: {} to boolean: {}.", left_value, right_value)),
                            _ => ()
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

    fn handle_check(&self, ast: &Vec<Node>, comparison_node: &ComparisonNode, mut childrens: Peekable<std::slice::Iter<Node>>) -> Result<(), String> {
        let comparison_type = &comparison_node.comparison;
        let left = &comparison_node.left;
        let right = &comparison_node.right;

        match comparison_type {
            TokenTypes::EqEq => compare!(self, childrens, ast, left, ==, right),
            TokenTypes::NotEq => compare!(self, childrens, ast, left, !=, right),
            TokenTypes::GThan => compare!(self, childrens, ast, left, >, right),
            TokenTypes::GThanEq => compare!(self, childrens, ast, left, >=, right),
            TokenTypes::LThan => compare!(self, childrens, ast, left, <, right),
            TokenTypes::LThanEq => compare!(self, childrens, ast, left, <=, right),
            _ => ()
        }

        Ok(())
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
            NodeTypes::Check(comparison_node, children) => {
                if let Err(err) = self.handle_check(ast, comparison_node, children.iter().peekable()) {
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

    fn find_variable(&self, mut nodes: Peekable<std::slice::Iter<Node>>, name: String) -> Result<VariableNode, ()> {
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
