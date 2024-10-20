use crate::parser::{ExprNodeKind, Node, NodeKind, VariableNode};
use std::iter::Peekable;
use inline_colorization::*;

pub fn run(ast: Vec<Node>) -> Result<(), String> {
    let mut nodes = ast.iter().peekable();

    while let Some(node) = nodes.next() {
        match &node.r#type {
            NodeKind::Log(args) => {
                let mut output = String::new();

                for arg in args {
                    match arg {
                        ExprNodeKind::String(value) => { output.push_str(value); },
                        ExprNodeKind::Int(value) => { output.push_str(value.to_string().as_str()); },
                        ExprNodeKind::Boolean(value) => { output.push_str(value.to_string().as_str()); },
                        ExprNodeKind::Identifier(name) => {
                            let result = find_variable(ast.iter().peekable(), name.to_string());
                            match &result {
                                Ok(variable) => match &variable.value {
                                    ExprNodeKind::String(str) => { output.push_str(str.as_str()); },
                                    ExprNodeKind::Int(int) => { output.push_str(int.to_string().as_str()); },
                                    ExprNodeKind::Boolean(value) => { output.push_str(value.to_string().as_str()); },
                                    _ => ()
                                },
                                Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", name))
                            }
                        }
                    }
                }

                print!("{output}");
            },
            NodeKind::Logl(args) => {
                let mut output = String::new();

                for arg in args {
                    match arg {
                        ExprNodeKind::String(value) => { output.push_str(value); },
                        ExprNodeKind::Int(value) => { output.push_str(value.to_string().as_str()); },
                        ExprNodeKind::Boolean(value) => { output.push_str(value.to_string().as_str()); },
                        ExprNodeKind::Identifier(name) => {
                            let result = find_variable(ast.iter().peekable(), name.to_string());
                            match &result {
                                Ok(variable) => match &variable.value {
                                    ExprNodeKind::String(str) => { output.push_str(str.as_str()); },
                                    ExprNodeKind::Int(int) => { output.push_str(int.to_string().as_str()); },
                                    ExprNodeKind::Boolean(value) => { output.push_str(value.to_string().as_str()); },
                                    _ => ()
                                },
                                Err(_) => return Err(format!("{color_red}[ERROR]{color_reset} -> Runtime Error: Unknown variable: '{}'.", name))
                            }
                        }
                    }
                }

                println!("{output}");
            },
            _ => {}
        }
    }

    Ok(())
}

fn find_variable(mut nodes: Peekable<std::slice::Iter<Node>>, name: String) -> Result<VariableNode, ()> {
    while let Some(node) = nodes.next() {
        match &node.r#type {
            NodeKind::Variable(variable) => {
                if variable.name == name {
                    return Ok(variable.clone());
                }
            },
            _ => {}
        }
    }

    Err(())
}
