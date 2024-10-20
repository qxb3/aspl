use std::{iter::Peekable, process::exit};

use crate::parser::{ExprNodeKind, Node, NodeKind, VariableNode};

pub fn run(ast: Vec<Node>) {
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
                            let variable = find_variable(ast.iter().peekable(), name.to_string());
                            match &variable.value {
                                ExprNodeKind::String(str) => { output.push_str(str); },
                                ExprNodeKind::Int(int) => { output.push_str(int.to_string().as_str()); },
                                ExprNodeKind::Boolean(value) => { output.push_str(value.to_string().as_str()); }
                                _ => {}
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
                            let variable = find_variable(ast.iter().peekable(), name.to_string());
                            match &variable.value {
                                ExprNodeKind::String(str) => { output.push_str(str); },
                                ExprNodeKind::Int(int) => { output.push_str(int.to_string().as_str()); },
                                ExprNodeKind::Boolean(value) => { output.push_str(value.to_string().as_str()); }
                                _ => {}
                            }
                        }
                    }
                }

                println!("{output}");
            },
            _ => {}
        }
    }
}

fn find_variable(mut nodes: Peekable<std::slice::Iter<Node>>, name: String) -> VariableNode {
    while let Some(node) = nodes.next() {
        match &node.r#type {
            NodeKind::Variable(variable) => {
                if variable.name == name {
                    return variable.clone();
                }
            },
            _ => {}
        }
    }

    println!("Error: Cannot find variable: {name}");
    exit(1);
}
