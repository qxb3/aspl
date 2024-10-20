use crate::lexer::{Token, TokenTypes};
use std::{iter::Peekable, process::exit};
use inline_colorization::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExprNodeTypes {
    Identifier(String),
    String(String),
    Int(i32),
    Boolean(bool)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VariableNode {
    pub name: String,
    pub value: ExprNodeTypes
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NodeTypes {
    Variable(VariableNode),
    Log(Vec<ExprNodeTypes>),
    Logl(Vec<ExprNodeTypes>)
}

#[derive(Debug, Clone)]
pub struct Node {
    pub r#type: NodeTypes
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
            Some(curr_token) if curr_token.r#type == TokenTypes::Identifier => { curr_token.value.as_ref().unwrap() },
            curr_token => {
                println!("{color_red}[ERROR]{color_reset}  -> Syntax Error: Wrong usage of 'set'");
                println!("{color_yellow}Position{color_reset} -> {}:{}", curr_token.unwrap().line, curr_token.unwrap().col);
                exit(1);
            }
        };

        tokens.next();

        let variable = match tokens.peek() {
            Some(curr_token) => match curr_token.r#type {
                TokenTypes::StringLiteral => VariableNode { name: identifier.to_string(), value: ExprNodeTypes::String(curr_token.value.clone().unwrap()) },
                TokenTypes::IntLiteral => VariableNode { name: identifier.to_string(), value: ExprNodeTypes::Int(curr_token.value.clone().unwrap().parse::<i32>().unwrap()) },
                TokenTypes::Boolean => VariableNode { name: identifier.to_string(), value: ExprNodeTypes::Boolean(curr_token.value.clone().unwrap().parse::<bool>().unwrap()) },
                _ => {
                    println!("{color_red}[ERROR]{color_reset}  -> Syntax Error: Unknown Identifier '{}'", curr_token.value.as_ref().unwrap());
                    println!("{color_yellow}Position{color_reset} -> {}:{}", curr_token.line, curr_token.col);
                    exit(1);
                }
            },
            curr_token => {
                println!("{color_red}[ERROR]{color_reset}  -> Syntax Error: Wrong usage of 'set'");
                println!("{color_yellow}Position{color_reset} -> {}:{}", curr_token.unwrap().line, curr_token.unwrap().col);
                exit(1);
            }
        };

        self.nodes.push(Node { r#type: NodeTypes::Variable(variable) });
    }

    fn parse_log(&mut self, tokens: &mut Peekable<std::slice::Iter<Token>>, command: &String) {
        let mut args: Vec<ExprNodeTypes> = Vec::new();

        while let Some(curr_token) = tokens.peek() {
            match curr_token.r#type {
                TokenTypes::Identifier => { args.push(ExprNodeTypes::Identifier(tokens.next().unwrap().value.clone().unwrap())); },
                TokenTypes::StringLiteral => { args.push(ExprNodeTypes::String(tokens.next().unwrap().value.clone().unwrap())); },
                TokenTypes::IntLiteral => { args.push(ExprNodeTypes::Int(tokens.next().unwrap().value.clone().unwrap().parse::<i32>().unwrap())); },
                TokenTypes::Boolean => { args.push(ExprNodeTypes::Boolean(tokens.next().unwrap().value.clone().unwrap().parse::<bool>().unwrap())); },
                _ => break
            }
        }

        match command.as_str() {
            "log" => { self.nodes.push(Node { r#type: NodeTypes::Log(args) }); },
            "logl" => { self.nodes.push(Node { r#type: NodeTypes::Logl(args) }); },
            _ => unreachable!()
        }
    }

    pub fn parse(&mut self, source_tokens: Vec<Token>) -> Vec<Node> {
        let mut tokens = source_tokens.iter().peekable();

        while let Some(token) = tokens.next() {
            match token {
                Token { r#type: TokenTypes::Command, value: Some(command), .. } => {
                    if command == "set" { self.parse_variable(&mut tokens); }
                    if command == "log" || command == "logl" { self.parse_log(&mut tokens, command); }
                },
                _ => ()
            }
        }

        self.nodes.clone()
    }
}
