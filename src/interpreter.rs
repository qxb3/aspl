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

    fn handle_check(&self, ast: &Vec<Node>, comparison_node: &ComparisonNode, mut children: Peekable<std::slice::Iter<Node>>) -> Result<(), String> {
        if let LiteralTypes::Int(left_value) = comparison_node.left {
            if let LiteralTypes::Int(right_value) = comparison_node.right {
                match comparison_node.comparison {
                    TokenTypes::EqEq => {
                        if left_value == right_value {
                            while let Some(curr_node) = children.next() {
                                if let Err(err) = self.execute_node(ast, curr_node) {
                                    return Err(err);
                                }
                            }
                        }
                    },
                    TokenTypes::GThan => {
                        if left_value > right_value {
                            while let Some(curr_node) = children.next() {
                                if let Err(err) = self.execute_node(ast, curr_node) {
                                    return Err(err);
                                }
                            }
                        }
                    },
                    TokenTypes::GThanEq => {
                        if left_value >= right_value {
                            while let Some(curr_node) = children.next() {
                                if let Err(err) = self.execute_node(ast, curr_node) {
                                    return Err(err);
                                }
                            }
                        }
                    },
                    TokenTypes::LThan => {
                        if left_value < right_value {
                            while let Some(curr_node) = children.next() {
                                if let Err(err) = self.execute_node(ast, curr_node) {
                                    return Err(err);
                                }
                            }
                        }
                    },
                    TokenTypes::LThanEq => {
                        if left_value <= right_value {
                            while let Some(curr_node) = children.next() {
                                if let Err(err) = self.execute_node(ast, curr_node) {
                                    return Err(err);
                                }
                            }
                        }
                    },
                    _ => unreachable!()
                };
            }
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
