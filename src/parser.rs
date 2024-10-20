use std::{iter::Peekable, process::exit};
use crate::lexer::{Token, TokenKind};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExprNodeKind {
    Identifier(String),
    String(String),
    Int(i32),
    Boolean(bool)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VariableNode {
    pub name: String,
    pub value: ExprNodeKind
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NodeKind {
    Variable(VariableNode),
    Log(Vec<ExprNodeKind>),
    Logl(Vec<ExprNodeKind>)
}

#[derive(Debug, Clone)]
pub struct Node {
    pub r#type: NodeKind
}

#[derive(Debug, Clone)]
pub struct Parser {
    nodes: Vec<Node>
}

impl Parser {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    fn parse_variable(&mut self, tokens: &mut Peekable<std::slice::Iter<Token>>) {
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
                TokenKind::Boolean => VariableNode { name: identifier.to_string(), value: ExprNodeKind::Boolean(curr_token.value.clone().unwrap().parse::<bool>().unwrap()) },
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

        self.nodes.push(Node { r#type: NodeKind::Variable(variable) });
    }

    fn parse_log(&mut self, tokens: &mut Peekable<std::slice::Iter<Token>>, command: &String) {
        let mut args: Vec<ExprNodeKind> = Vec::new();

        while let Some(curr_token) = tokens.peek() {
            match curr_token.r#type {
                TokenKind::Identifier => { args.push(ExprNodeKind::Identifier(tokens.next().unwrap().value.clone().unwrap())); },
                TokenKind::StringLiteral => { args.push(ExprNodeKind::String(tokens.next().unwrap().value.clone().unwrap())); },
                TokenKind::IntLiteral => { args.push(ExprNodeKind::Int(tokens.next().unwrap().value.clone().unwrap().parse::<i32>().unwrap())); },
                TokenKind::Boolean => { args.push(ExprNodeKind::Boolean(tokens.next().unwrap().value.clone().unwrap().parse::<bool>().unwrap())); },
                _ => break
            }
        }

        match command.as_str() {
            "log" => { self.nodes.push(Node { r#type: NodeKind::Log(args) }); },
            "logl" => { self.nodes.push(Node { r#type: NodeKind::Logl(args) }); },
            _ => unreachable!()
        }
    }

    pub fn parse(&mut self, source_tokens: Vec<Token>) -> Vec<Node> {
        let mut tokens = source_tokens.iter().peekable();

        while let Some(token) = tokens.next() {
            match token {
                Token { r#type: TokenKind::Command, value: Some(command) } => {
                    if command == "set" { self.parse_variable(&mut tokens); }
                    if command == "log" || command == "logl" { self.parse_log(&mut tokens, command); }
                },
                _ => ()
            }
        }

        self.nodes.clone()
    }
}
