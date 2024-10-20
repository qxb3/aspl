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
    pub r#type: NodeKind
}

pub fn parse(source_tokens: Vec<Token>) -> Vec<Node> {
    let mut nodes: Vec<Node> = Vec::new();
    let mut tokens = source_tokens.iter().peekable();

    while let Some(token) = tokens.next() {
        if token.r#type == TokenKind::Command && token.value.as_ref().unwrap() == "set" {
            let identifier = match tokens.peek() {
                Some(curr_token) => if curr_token.r#type == TokenKind::Identifier {
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
                Some(curr_token) => match curr_token.r#type {
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

            nodes.push(Node { r#type: NodeKind::Variable(variable) });
        }

        match token.value.as_ref().unwrap().as_str() {
            log_type if log_type == "log" || log_type == "logl" => {
                let mut args: Vec<ExprNodeKind> = Vec::new();

                while let Some(curr_token) = tokens.peek() {
                    match curr_token.r#type {
                        TokenKind::Identifier => { args.push(ExprNodeKind::Identifier(tokens.next().unwrap().value.clone().unwrap())); },
                        TokenKind::StringLiteral => { args.push(ExprNodeKind::String(tokens.next().unwrap().value.clone().unwrap())); },
                        TokenKind::IntLiteral => { args.push(ExprNodeKind::String(tokens.next().unwrap().value.clone().unwrap())); },
                        _ => break
                    }
                }

                match log_type {
                    "log" => { nodes.push(Node { r#type: NodeKind::Log(args) }); },
                    "logl" => { nodes.push(Node { r#type: NodeKind::Logl(args) }); },
                    _ => unreachable!()
                }
            },
            _ => unreachable!()
        }
    }

    nodes
}
