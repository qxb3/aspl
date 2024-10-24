use crate::lexer::{Token, TokenTypes};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Literals {
    String(String),
    Int(i32),
    Boolean(bool)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Node {
    Literal(Literals),
    Identifier(String),
    Var {
        identifier: Box<Node>,
        value: Literals
    },
    Condition {
        left: Box<Node>,
        condition: String,
        right: Box<Node>
    },
    Scope {
        body: Vec<Box<Node>>
    },

    // Commands
    Log {
        r#type: String,
        args: Vec<Box<Node>>
    },
    Check {
        condition: Box<Node>,
        scope: Box<Node>
    },
    While {
        condition: Box<Node>,
        scope: Box<Node>
    }
}

#[derive(Debug, Clone)]
pub struct Parser<T: Iterator<Item = Token> + Clone> {
    tokens: T,
    current_token: Option<Token>
}

impl<T: Iterator<Item = Token> + Clone> Parser<T> {
    pub fn new(mut tokens: T) -> Self {
        let current_token = tokens.next();
        Self { tokens, current_token }
    }

    fn parse_set_command(&mut self) -> Result<Node, String> {
        self.advance();

        let identifier = self.parse_identifier()?;

        let value = match &self.current_token {
            Some(node) => match node.r#type {
                TokenTypes::IntLiteral => Literals::Int(node.value.clone().unwrap().parse().unwrap()),
                TokenTypes::StringLiteral => Literals::String(node.value.clone().unwrap().parse().unwrap()),
                TokenTypes::BooleanLiteral => Literals::Boolean(node.value.clone().unwrap().parse().unwrap()),
                _ => return Err(format!("Expected a literal, but found {:?}", node.r#type))
            },
            None => return Err(format!("Unexpected end of input while parsing set command"))
        };

        self.advance();

        Ok(Node::Var {
            identifier: Box::new(identifier),
            value
        })
    }

    fn parse_log_command(&mut self, command: String) -> Result<Node, String> {
        self.advance();

        let mut args: Vec<Box<Node>> = vec![];

        while let Some(arg) = &self.current_token {
            match arg.r#type {
                TokenTypes::IntLiteral |
                TokenTypes::StringLiteral |
                TokenTypes::BooleanLiteral => {
                    let literal = self.parse_literal()?;
                    args.push(Box::new(literal));
                },
                TokenTypes::Identifier => {
                    let var = self.parse_identifier()?;
                    args.push(Box::new(var));
                },
                _ => break
            }
        }

        if args.is_empty() {
            return Err(format!("Unexpected end of input while parsing {} command", command))
        }

        Ok(Node::Log {
            r#type: command,
            args
        })
    }

    fn parse_check_command(&mut self) -> Result<Node, String> {
        self.advance();

        let condition = self.parse_condition()?;
        let scope = self.parse_scope()?;

        Ok(Node::Check {
            condition: Box::new(condition),
            scope: Box::new(scope)
        })
    }

    fn parse_while_command(&mut self) -> Result<Node, String> {
        self.advance();

        let condition = self.parse_condition()?;
        let scope = self.parse_scope()?;

        Ok(Node::While {
            condition: Box::new(condition),
            scope: Box::new(scope)
        })
    }

    fn parse_command(&mut self) -> Result<Node, String> {
        if let Some(token) = &self.current_token {
            let command = token.value.clone().unwrap();

            match command.as_str() {
                "set"           =>  return self.parse_set_command(),
                "log" | "logl"  =>  return self.parse_log_command(command),
                "check"         =>  return self.parse_check_command(),
                "while"         =>  return self.parse_while_command(),
                _ =>                return Err(format!("Expected a command, but found {:?}", token.r#type))
            }
        }

        return Err("Unexpected end of input while parsing command".to_string());
    }

    fn parse_literal(&mut self) -> Result<Node, String> {
        if let Some(token) = &self.current_token {
            let value: Option<Literals> = match token.r#type {
                TokenTypes::IntLiteral => Some(Literals::Int(token.value.clone().unwrap().parse().unwrap())),
                TokenTypes::StringLiteral => Some(Literals::String(token.value.clone().unwrap().parse().unwrap())),
                TokenTypes::BooleanLiteral => Some(Literals::Boolean(token.value.clone().unwrap().parse().unwrap())),
                _ => None
            };

            if value.is_none() {
                return Err(format!(
                    "Expected a literal, but found {:?}",
                    token.r#type
                ));
            }

            self.advance();
            return Ok(Node::Literal(value.unwrap()))
        }

        return Err(format!("Unexpected end of input while parsing literal"));
    }

    fn parse_identifier(&mut self) -> Result<Node, String> {
        if let Some(token) = &self.current_token.clone() {
            if !token.r#type.is_identifier() {
                return Err(format!("Expected a identifier, but found {:?}", token.r#type));
            }

            self.advance();

            return Ok(Node::Identifier(token.value.clone().unwrap()));
        }

        return Err(format!("Unexpected end of input while parsing identifier"))
    }

    fn parse_condition(&mut self) -> Result<Node, String> {
        let left = self.parse_literal()?;

        let condition = match &self.current_token.clone().unwrap().r#type {
            TokenTypes::EqEq => "==",
            TokenTypes::NotEq => "!=",
            TokenTypes::GThan => ">",
            TokenTypes::GThanEq => ">=",
            TokenTypes::LThan => "<",
            TokenTypes::LThanEq => "<=",
            token_type => return Err(format!("Expected a condition, but found {:?}", token_type))
        };

        self.advance();

        let right = self.parse_literal()?;

        Ok(Node::Condition {
            left: Box::new(left),
            condition: condition.to_string(),
            right: Box::new(right)
        })
    }

    fn parse_scope(&mut self) -> Result<Node, String> {
        self.advance();

        let mut body: Vec<Box<Node>> = vec![];

        while let Some(token) = &self.current_token {
            if token.r#type.is_close_curly() {
                self.advance();
                break;
            }

            let parsed_token = self.parse_token()?;
            body.push(Box::new(parsed_token));
        }

        Ok(Node::Scope {
            body
        })
    }

    // Parse all expressions
    fn parse_expr(&mut self) -> Result<Node, String> {
        if let Some(token) = &self.current_token {
            if token.r#type.is_literal() || token.r#type.is_identifier() {
                // Check & Parse Condition
                if let Some(condition) = self.peek() {
                    if condition.r#type.is_condition_op() {
                        return self.parse_condition();
                    }
                }
            }

            // Check & Parse Literal
            if token.r#type.is_literal() {
                return self.parse_literal();
            }

            // Check & Parse Identifier
            if token.r#type.is_identifier() {
                return self.parse_identifier();
            }

            // Check & Parse Scope
            if token.r#type.is_open_curly() {
                return self.parse_scope();
            }
        }

        Err(format!("Unexpected end of input while parsing expression"))
    }

    fn parse_token(&mut self) -> Result<Node, String> {
        if let Some(token) = &self.current_token {
            if token.r#type.is_command() {
                return self.parse_command();
            } else {
                return self.parse_expr();
            }
        }

        Err(format!("Unhandled Token {:?}", &self.current_token))
    }

    pub fn parse(&mut self) -> Result<Vec<Node>, String> {
        let mut ast = Vec::new();

        while let Some(_) = &self.current_token {
            let parsed_token = self.parse_token()?;
            ast.push(parsed_token);
        }

        Ok(ast)
    }

    fn advance(&mut self) {
        self.current_token = self.tokens.next();
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.clone().next()
    }
}
