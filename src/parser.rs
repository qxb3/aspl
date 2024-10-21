use crate::lexer::{Token, TokenTypes};
use std::{iter::Peekable, process::exit};
use inline_colorization::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LiteralTypes {
    String(String),
    Int(i32),
    Boolean(bool)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExprNodeTypes {
    Identifier(String),
    Literal(LiteralTypes)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VariableNode {
    pub name: String,
    pub value: ExprNodeTypes
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ComparisonNode {
    pub left: ExprNodeTypes,
    pub right: ExprNodeTypes,
    pub comparison: TokenTypes
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NodeTypes {
    Variable(VariableNode),
    Log(Vec<ExprNodeTypes>),
    Logl(Vec<ExprNodeTypes>),
    Check(ComparisonNode, Vec<Node>)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub r#type: NodeTypes
}

#[derive(Debug, Clone)]
pub struct Parser {
    nodes: Vec<Node>,
}

impl Parser {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    fn parse_variable(&mut self, tokens: &mut Peekable<std::slice::Iter<Token>>) -> Option<Node> {
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
                TokenTypes::StringLiteral => VariableNode { name: identifier.to_string(), value: ExprNodeTypes::Literal(LiteralTypes::String(curr_token.value.clone().unwrap())) },
                TokenTypes::IntLiteral => VariableNode { name: identifier.to_string(), value: ExprNodeTypes::Literal(LiteralTypes::Int(curr_token.value.clone().unwrap().parse::<i32>().unwrap())) },
                TokenTypes::Boolean => VariableNode { name: identifier.to_string(), value: ExprNodeTypes::Literal(LiteralTypes::Boolean(curr_token.value.clone().unwrap().parse::<bool>().unwrap())) },
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

        Some(Node { r#type: NodeTypes::Variable(variable) })
    }

    fn parse_log(&mut self, tokens: &mut Peekable<std::slice::Iter<Token>>, command: &String) -> Option<Node> {
        let mut args: Vec<ExprNodeTypes> = Vec::new();

        while let Some(curr_token) = tokens.peek() {
            match curr_token.r#type {
                TokenTypes::Identifier => { args.push(ExprNodeTypes::Identifier(tokens.next().unwrap().value.clone().unwrap())); },
                TokenTypes::StringLiteral => { args.push(ExprNodeTypes::Literal(LiteralTypes::String(tokens.next().unwrap().value.clone().unwrap()))); },
                TokenTypes::IntLiteral => { args.push(ExprNodeTypes::Literal(LiteralTypes::Int(tokens.next().unwrap().value.clone().unwrap().parse::<i32>().unwrap()))); },
                TokenTypes::Boolean => { args.push(ExprNodeTypes::Literal(LiteralTypes::Boolean(tokens.next().unwrap().value.clone().unwrap().parse::<bool>().unwrap()))); },
                _ => break
            }
        }

        match command.as_str() {
            "log" => Some( Node { r#type: NodeTypes::Log(args) } ),
            "logl" => Some( Node { r#type: NodeTypes::Logl(args) } ),
            _ => None
        }
    }

    fn parse_comparison(&mut self, tokens: &mut Peekable<std::slice::Iter<Token>>, left: &Token, comparison_type: &Token, right: &Token) -> Option<Node> {
        let mut childrens: Vec<Node> = Vec::new();

        if tokens.peek().unwrap().r#type == TokenTypes::OpenCurly {
            tokens.next();

            while let Some(curr_token) = tokens.clone().peek() {
                if curr_token.r#type == TokenTypes::CloseCurly { break; }

                tokens.next();
                if let Some(parsed_token) = self.parse_token(tokens, curr_token) {
                    childrens.push(parsed_token);
                }
            }
        }

        let get_expr = |token: &Token| {
            match token.r#type {
                TokenTypes::IntLiteral => ExprNodeTypes::Literal(LiteralTypes::Int(token.value.as_ref().unwrap().parse::<i32>().unwrap())),
                TokenTypes::StringLiteral => ExprNodeTypes::Literal(LiteralTypes::String(token.value.as_ref().unwrap().to_string())),
                TokenTypes::Boolean => ExprNodeTypes::Literal(LiteralTypes::Boolean(token.value.as_ref().unwrap().parse::<bool>().unwrap())),
                TokenTypes::Identifier => ExprNodeTypes::Identifier(token.value.as_ref().unwrap().to_string()),
                _ => {
                    println!("{color_red}[ERROR]{color_reset}  -> Syntax Error: Wrong usage of 'check'");
                    println!("{color_yellow}Position{color_reset} -> {}:{}", token.line, token.col);
                    exit(1);
                }
            }
        };

        return Some(Node {
            r#type: NodeTypes::Check(ComparisonNode {
                left: get_expr(left),
                right: get_expr(right),
                comparison: comparison_type.r#type
            }, childrens
        )})
    }

    fn parse_check(&mut self, tokens: &mut Peekable<std::slice::Iter<Token>>) -> Option<Node> {
        if let Some(&left) = tokens.peek() {
            if  left.r#type == TokenTypes::IntLiteral ||
                left.r#type == TokenTypes::StringLiteral ||
                left.r#type == TokenTypes::Boolean ||
                left.r#type == TokenTypes::Identifier {
                tokens.next();

                if let Some(&comparison_type) = tokens.peek() {
                    if  comparison_type.r#type == TokenTypes::EqEq ||
                        comparison_type.r#type == TokenTypes::NotEq ||
                        comparison_type.r#type == TokenTypes::GThan ||
                        comparison_type.r#type == TokenTypes::GThanEq ||
                        comparison_type.r#type == TokenTypes::LThan ||
                        comparison_type.r#type == TokenTypes::LThanEq {
                        tokens.next();

                        if let Some(&right) = tokens.peek() {
                            if  right.r#type == TokenTypes::IntLiteral ||
                                right.r#type == TokenTypes::StringLiteral ||
                                right.r#type == TokenTypes::Boolean ||
                                right.r#type == TokenTypes::Identifier {
                                tokens.next();

                                if let Some(node) = self.parse_comparison(tokens, left, comparison_type, right) {
                                    return Some(node);
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn parse_token(&mut self, tokens: &mut Peekable<std::slice::Iter<Token>>, token: &Token) -> Option<Node> {
        match token {
            Token { r#type: TokenTypes::Command, value: Some(command), .. } => {
                match command.as_str() {
                    "set"           => Some(self.parse_variable(tokens)).unwrap(),
                    "log" | "logl"  => Some(self.parse_log(tokens, command)).unwrap(),
                    "check"         => Some(self.parse_check(tokens)).unwrap(),
                    _ => None
                }
            },
            _ => None
        }
    }

    pub fn parse(&mut self, source_tokens: Vec<Token>) -> Vec<Node> {
        let mut tokens = source_tokens.iter().peekable();

        while let Some(token) = tokens.next() {
            if let Some(node) = self.parse_token(&mut tokens, token) {
                self.nodes.push(node);
            }
        }

        self.nodes.clone()
    }
}
