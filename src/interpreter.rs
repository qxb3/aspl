use std::{iter::Peekable, process::exit};

use crate::parser::{ExprNodeKind, Node, NodeKind, VariableNode};

pub fn run(ast: Vec<Node>) {
    let mut nodes = ast.iter().peekable();

    while let Some(node) = nodes.next() {
        match &node.r#type {
            NodeKind::Log(args) => {
                for arg in args {
                    match arg {
                        ExprNodeKind::String(value) => { print!("{value}"); },
                        ExprNodeKind::Int(value) => { print!("{value}"); },
                        ExprNodeKind::Identifier(name) => {
                            let variable = find_variable(ast.iter().peekable(), name.to_string());
                            match &variable.value {
                                ExprNodeKind::String(str) => { print!("{str}"); },
                                ExprNodeKind::Int(int) => { print!("{int}"); },
                                _ => {}
                            }
                        }
                    }
                }
            },
            NodeKind::Logl(args) => {
                for arg in args {
                    match arg {
                        ExprNodeKind::String(value) => { println!("{value}"); },
                        ExprNodeKind::Int(value) => { println!("{value}"); },
                        ExprNodeKind::Identifier(name) => {
                            let variable = find_variable(ast.iter().peekable(), name.to_string());
                            match &variable.value {
                                ExprNodeKind::String(str) => { println!("{str}"); },
                                ExprNodeKind::Int(int) => { println!("{int}"); },
                                _ => {}
                            }
                        }
                    }
                }
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
