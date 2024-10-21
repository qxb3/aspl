use crate::{lexer::TokenTypes, parser::{ComparisonNode, ExprNodeTypes, LiteralTypes, Node, NodeTypes, VariableNode}};
use std::iter::Peekable;
use inline_colorization::*;

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
        let handle_comparison_int = |left_value: &i32, right_value: &i32, comparison: &TokenTypes, childrens: &mut Peekable<std::slice::Iter<'_, Node>>| {
            match comparison {
                TokenTypes::EqEq => {
                    if left_value == right_value {
                        while let Some(curr_node) = childrens.next() {
                            if let Err(err) = self.execute_node(ast, curr_node) {
                                return Err(err);
                            }
                        }
                    }
                },
                TokenTypes::GThan => {
                    if left_value > right_value {
                        while let Some(curr_node) = childrens.next() {
                            if let Err(err) = self.execute_node(ast, curr_node) {
                                return Err(err);
                            }
                        }
                    }
                },
                TokenTypes::GThanEq => {
                    if left_value >= right_value {
                        while let Some(curr_node) = childrens.next() {
                            if let Err(err) = self.execute_node(ast, curr_node) {
                                return Err(err);
                            }
                        }
                    }
                },
                TokenTypes::LThan => {
                    if left_value < right_value {
                        while let Some(curr_node) = childrens.next() {
                            if let Err(err) = self.execute_node(ast, curr_node) {
                                return Err(err);
                            }
                        }
                    }
                },
                TokenTypes::LThanEq => {
                    if left_value <= right_value {
                        while let Some(curr_node) = childrens.next() {
                            if let Err(err) = self.execute_node(ast, curr_node) {
                                return Err(err);
                            }
                        }
                    }
                },
                _ => {}
            }

            Ok(())
        };

        match comparison_node {
            ComparisonNode { left: ExprNodeTypes::Literal(LiteralTypes::Int(left_value)), right: ExprNodeTypes::Literal(LiteralTypes::Int(right_value)), comparison } => {
                if let Err(err) = handle_comparison_int(left_value, right_value, comparison, &mut childrens) {
                    return Err(err);
                }
            },
            ComparisonNode { left: ExprNodeTypes::Identifier(left_name), right: ExprNodeTypes::Literal(LiteralTypes::Int(right_value)), comparison } => {
                let left_result = self.find_variable(ast.iter().peekable(), left_name.to_string());

                match &left_result {
                    Ok(left_variable) => match &left_variable.value {
                        ExprNodeTypes::Literal(LiteralTypes::Int(left_value)) => {
                            if let Err(err) = handle_comparison_int(left_value, right_value, comparison, &mut childrens) {
                                return Err(err);
                            }
                        },
                        ExprNodeTypes::Literal(LiteralTypes::String(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare string: \"{}\" to int: {}.", left_value, right_value)),
                        ExprNodeTypes::Literal(LiteralTypes::Boolean(left_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare boolean: {} to int: {}.", left_value, right_value)),
                        _ => ()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", left_name))
                }
            },
            ComparisonNode { left: ExprNodeTypes::Literal(LiteralTypes::Int(left_value)), right: ExprNodeTypes::Identifier(right_name), comparison } => {
                let right_result = self.find_variable(ast.iter().peekable(), right_name.to_string());

                match &right_result {
                    Ok(right_variable) => match &right_variable.value {
                        ExprNodeTypes::Literal(LiteralTypes::Int(right_value)) => {
                            if let Err(err) = handle_comparison_int(left_value, right_value, comparison, &mut childrens) {
                                return Err(err);
                            }
                        },
                        ExprNodeTypes::Literal(LiteralTypes::String(right_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot compare int: {} to string: \"{}\".", left_value, right_value)),
                        ExprNodeTypes::Literal(LiteralTypes::Boolean(right_value)) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Cannot int: {} to compare boolean: {}.", left_value, right_value)),
                        _ => ()
                    },
                    Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", right_name))
                }
            },
            ComparisonNode { left: ExprNodeTypes::Identifier(left_name), right: ExprNodeTypes::Identifier(right_name), comparison } => {
                let left_result = self.find_variable(ast.iter().peekable(), left_name.to_string());
                let right_result = self.find_variable(ast.iter().peekable(), right_name.to_string());

                match (&left_result, &right_result) {
                    (Ok(left_variable), Ok(right_variable)) => {
                        match (&left_variable.value, &right_variable.value) {
                            (ExprNodeTypes::Literal(LiteralTypes::Int(left_value)), ExprNodeTypes::Literal(LiteralTypes::Int(right_value))) => {
                                if let Err(err) = handle_comparison_int(left_value, right_value, comparison, &mut childrens) {
                                    return Err(err);
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
            _ => {}
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
