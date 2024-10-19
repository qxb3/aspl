use std::process::exit;

use crate::lexer::{Token, TokenKind};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExprNodeKind {
    Identifier(String),
    String(String),
    Int(i32)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VariableNode {
    pub name: String,
    pub value: ExprNodeKind
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeKind {
    Variable(VariableNode),
    Log(Vec<ExprNodeKind>),
    Logl(Vec<ExprNodeKind>)
}

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind
}

pub fn parse(source_tokens: Vec<Token>) -> Vec<Node> {
    let mut nodes: Vec<Node> = Vec::new();
    let mut tokens = source_tokens.iter().peekable();

    while let Some(token) = tokens.next() {
        if token.kind == TokenKind::Command && token.value.as_ref().unwrap() == "set" {

            let identifier = match tokens.peek() {
                Some(curr_token) => if curr_token.kind == TokenKind::Identifier {
                    curr_token.value.as_ref().unwrap()
                } else {
                    println!("Syntax error on command: set");
                    exit(1);
                },
                None => {
                    println!("Syntax error on command: set");
                    exit(1);
                }
            };

            tokens.next();

            let variable = match tokens.peek() {
                Some(curr_token) => match curr_token.kind {
                    TokenKind::StringLiteral => VariableNode { name: identifier.to_string(), value: ExprNodeKind::String(curr_token.value.clone().unwrap()) },
                    TokenKind::IntLiteral => VariableNode { name: identifier.to_string(), value: ExprNodeKind::Int(curr_token.value.clone().unwrap().parse::<i32>().unwrap()) },
                    _ => {
                        println!("Syntax error on command: set");
                        exit(1);
                    }
                },
                None => {
                    println!("Syntax error on command: set");
                    exit(1);
                }
            };

            nodes.push(Node { kind: NodeKind::Variable(variable) });
        }

        let log_type = token.value.as_ref().unwrap();
        if token.kind == TokenKind::Command && (log_type == "log" || log_type == "logl") {
            let mut args: Vec<ExprNodeKind> = Vec::new();

            while let Some(curr_token) = tokens.peek() {
                if curr_token.kind == TokenKind::Identifier {
                    args.push(ExprNodeKind::Identifier(tokens.next().unwrap().value.clone().unwrap()));
                } else if curr_token.kind == TokenKind::StringLiteral {
                    args.push(ExprNodeKind::String(tokens.next().unwrap().value.clone().unwrap()));
                } else if curr_token.kind == TokenKind::IntLiteral {
                    args.push(ExprNodeKind::String(tokens.next().unwrap().value.clone().unwrap()));
                } else {
                    break;
                }
            }

            if log_type == "log" {
                nodes.push(Node { kind: NodeKind::Log(args) });
            } else if log_type == "logl" {
                nodes.push(Node { kind: NodeKind::Logl(args) });
            }
        }
    }

    nodes
}
