use crate::lexer::{Token, TokenTypes};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Literals {
    String(String),
    Int(i64),
    Boolean(bool)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Node {
    Literal(Literals),
    Identifier(String),
    Return(Box<Node>),
    Var {
        identifier: Box<Node>,
        value: Box<Node>
    },
    Condition {
        left: Box<Node>,
        condition: String,
        right: Box<Node>
    },
    Scope {
        body: Vec<Box<Node>>
    },
    Function {
        identifier: Box<Node>,
        args: Vec<Box<Node>>,
        scope: Box<Node>
    },
    FunctionCall {
        identifier: Box<Node>,
        args: Vec<Box<Node>>
    },

    // Statements
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

#[derive(Debug)]
pub struct ParserError {
    pub message: String,
    pub token: Option<Token>
}

type ParserResult<T> = Result<T, ParserError>;

impl<T: Iterator<Item = Token> + Clone> Parser<T> {
    pub fn new(mut tokens: T) -> Self {
        let current_token = tokens.next();
        Self { tokens, current_token }
    }

    fn parse_set_statement(&mut self) -> ParserResult<Node> {
        self.advance();

        let identifier = self.parse_identifier()?;

        let value = match &self.current_token {
            Some(node) => match node.r#type {
                TokenTypes::IntLiteral => Node::Literal(Literals::Int(node.value.clone().unwrap().parse().unwrap())),
                TokenTypes::StringLiteral => Node::Literal(Literals::String(node.value.clone().unwrap().parse().unwrap())),
                TokenTypes::BooleanLiteral => Node::Literal(Literals::Boolean(node.value.clone().unwrap().parse().unwrap())),
                TokenTypes::Identifier => self.parse_identifier()?,
                TokenTypes::FnCall => self.parse_function_call()?,
                _ => return Err(ParserError {
                    message: format!("Expected a literal/identifier/function call, but found {:?}", node.r#type),
                    token: Some(node.clone())
                })
            },
            None => return Err(ParserError {
                message: format!("Unexpected end of input while parsing set statement"),
                token: None
            })
        };

        Ok(Node::Var {
            identifier: Box::new(identifier),
            value: Box::new(value)
        })
    }

    fn parse_log_statement(&mut self, statement: String) -> ParserResult<Node> {
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
                TokenTypes::FnCall => {
                    let fn_call = self.parse_function_call()?;
                    args.push(Box::new(fn_call));
                },
                _ => break
            }
        }

        if args.is_empty() {
            return Err(ParserError {
                message: format!("Unexpected end of input while parsing {} statement", statement),
                token: None
            });
        }

        Ok(Node::Log {
            r#type: statement,
            args
        })
    }

    fn parse_check_statement(&mut self) -> ParserResult<Node> {
        self.advance();

        if let Some(token) = &self.current_token {
            if token.r#type.is_literal() || token.r#type.is_identifier() {
                if let Some(token) = self.peek() {
                    if token.r#type.is_condition_op() {
                        let condition = self.parse_condition()?;
                        let scope = self.parse_scope()?;

                        return Ok(Node::Check {
                            condition: Box::new(condition),
                            scope: Box::new(scope)
                        })
                    }
                }
            }

            if token.r#type.is_literal() {
                let literal = self.parse_literal()?;
                let scope = self.parse_scope()?;

                return Ok(Node::Check {
                    condition: Box::new(literal),
                    scope: Box::new(scope)
                });
            }
        }

        Err(ParserError {
            message: format!("Unexpected end of input while parsing check statement"),
            token: None
        })
    }

    fn parse_while_statement(&mut self) -> ParserResult<Node> {
        self.advance();

        if let Some(token) = &self.current_token {
            if token.r#type.is_literal() || token.r#type.is_identifier() {
                if let Some(condition) = self.peek() {
                    if condition.r#type.is_condition_op() {
                        let condition = self.parse_condition()?;
                        let scope = self.parse_scope()?;

                        return Ok(Node::While {
                            condition: Box::new(condition),
                            scope: Box::new(scope)
                        })
                    }
                }
            }

            if token.r#type.is_literal() {
                let literal = self.parse_literal()?;
                let scope = self.parse_scope()?;

                return Ok(Node::While {
                    condition: Box::new(literal),
                    scope: Box::new(scope)
                });
            }

            return Err(ParserError {
                message: format!("Expected a condition or literal, but found {:?}", token),
                token: Some(token.clone())
            })
        }

        Err(ParserError {
            message: format!("Unexpected end of input while parsing while statement"),
            token: None
        })
    }

    fn parse_function(&mut self) -> ParserResult<Node>{
        self.advance();

        let identifier = self.parse_identifier()?;
        let mut args: Vec<Box<Node>> = vec![];

        while let Some(token) = &self.current_token {
            if !token.r#type.is_identifier() {
                break;
            }

            if let Ok(arg) = self.parse_identifier() {
                args.push(Box::new(arg));
            }
        }

        let scope = self.parse_scope()?;

        Ok(Node::Function {
            identifier: Box::new(identifier.clone()),
            args,
            scope: Box::new(scope)
        })
    }

    fn parse_return(&mut self) -> ParserResult<Node> {
        self.advance();

        if let Some(token) = &self.current_token {
            if token.r#type.is_literal() {
                let ret_identifier = self.parse_literal()?;
                return Ok(Node::Return(
                    Box::new(ret_identifier)
                ))
            }

            if token.r#type.is_identifier() {
                let ret_literal = self.parse_identifier()?;
                return Ok(Node::Return(
                    Box::new(ret_literal)
                ))
            }

            if token.r#type.is_fn_call() {
                let ret_fn_call = self.parse_function_call()?;
                return Ok(Node::Return(
                    Box::new(ret_fn_call)
                ))
            }

            return Err(ParserError {
                message: format!("Expected a literal/identifier/fn_call, but found {:?}", token.r#type),
                token: Some(token.clone())
            });
        }

        Err(ParserError {
            message: format!("Unexpected end of input while parsing return"),
            token: None
        })
    }

    // Parse all statements
    fn parse_statement(&mut self) -> ParserResult<Node> {
        if let Some(token) = &self.current_token {
            let statement = token.value.clone().unwrap();

            match statement.as_str() {
                "set"           =>  return self.parse_set_statement(),
                "log" | "logl"  =>  return self.parse_log_statement(statement),
                "check"         =>  return self.parse_check_statement(),
                "while"         =>  return self.parse_while_statement(),
                "fn"            =>  return self.parse_function(),
                "ret"           =>  return self.parse_return(),

                _ =>  return Err(ParserError {
                    message: format!("Expected a statement, but found {:?}", token.r#type),
                    token: Some(token.clone())
                })
            }
        }

        Err(ParserError {
            message: format!("Unexpected end of input while parsing statement"),
            token: None
        })
    }

    fn parse_literal(&mut self) -> ParserResult<Node> {
        if let Some(token) = &self.current_token {
            let value: Option<Literals> = match token.r#type {
                TokenTypes::IntLiteral => Some(Literals::Int(token.value.clone().unwrap().parse().unwrap())),
                TokenTypes::StringLiteral => Some(Literals::String(token.value.clone().unwrap().parse().unwrap())),
                TokenTypes::BooleanLiteral => Some(Literals::Boolean(token.value.clone().unwrap().parse().unwrap())),
                _ => None
            };

            if value.is_none() {
                return Err(ParserError {
                    message: format!("Expected a literal, but found {:?}", token.r#type),
                    token: Some(token.clone())
                });
            }

            self.advance();
            return Ok(Node::Literal(value.unwrap()))
        }

        Err(ParserError {
            message: format!("Unexpected end of input while parsing literal"),
            token: None
        })
    }

    fn parse_identifier(&mut self) -> ParserResult<Node> {
        if let Some(token) = &self.current_token.clone() {
            if !token.r#type.is_identifier() {
                return Err(ParserError {
                    message: format!("Expected a identifier, but found {:?}", token.r#type),
                    token: Some(token.clone())
                });
            }

            self.advance();

            return Ok(Node::Identifier(token.value.clone().unwrap()));
        }

        Err(ParserError {
            message: format!("Unexpected end of input while parsing identifier"),
            token: None
        })
    }

    fn parse_condition(&mut self) -> ParserResult<Node> {
        let left = match &self.current_token {
            Some(left) => match left {
                left if left.r#type.is_identifier() => self.parse_identifier()?,
                left if left.r#type.is_literal() => self.parse_literal()?,
                left => return Err(ParserError {
                    message: format!("Expected a identifier or literal, but found {:?}", left),
                    token: Some(left.clone())
                })
            },
            None => return Err(ParserError {
                message: format!("Unexpected end of input while parsing condition"),
                token: None
            })
        };

        let condition = match &self.current_token {
            Some(token) => match token.r#type {
                TokenTypes::EqEq => "==",
                TokenTypes::NotEq => "!=",
                TokenTypes::GThan => ">",
                TokenTypes::GThanEq => ">=",
                TokenTypes::LThan => "<",
                TokenTypes::LThanEq => "<=",
                token_type => return Err(ParserError {
                    message: format!("Expected a condition, but found {:?}", token_type),
                    token: Some(token.clone())
                })
            },
            None => return Err(ParserError {
                message: format!("Unexpected end of input while parsing condition"),
                token: None
            })
        };

        // Advance from condition
        self.advance();

        let right = match &self.current_token {
            Some(right) => match right {
                right if right.r#type.is_identifier() => self.parse_identifier()?,
                right if right.r#type.is_literal() => self.parse_literal()?,
                right => return Err(ParserError {
                    message: format!("Expected a identifier or literal, but found {:?}", right),
                    token: Some(right.clone())
                })
            },
            None => return Err(ParserError {
                message: format!("Unexpected end of input while parsing condition"),
                token: None
            })
        };

        Ok(Node::Condition {
            left: Box::new(left),
            condition: condition.to_string(),
            right: Box::new(right)
        })
    }

    fn parse_scope(&mut self) -> ParserResult<Node> {
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

    fn parse_function_call(&mut self) -> ParserResult<Node> {
        let identifier = match &self.current_token {
            Some(token) => Node::Identifier(token.clone().value.unwrap_or_default()),
            None => return Err(ParserError {
                message: format!("Unexpected end of input while parsing function call"),
                token: None
            })
        };

        self.advance();

        let mut args: Vec<Box<Node>> = vec![];

        while let Some(token) = &self.current_token {
            let foo = match token.r#type {
                r#type if r#type.is_literal() => self.parse_literal()?,
                r#type if r#type.is_identifier() => self.parse_identifier()?,
                _ => break
            };

            args.push(Box::new(foo));
        }

        Ok(Node::FunctionCall {
            identifier: Box::new(identifier),
            args
        })
    }

    // Parse all expressions
    fn parse_expr(&mut self) -> ParserResult<Node> {
        if let Some(token) = &self.current_token {
            if token.r#type.is_literal() || token.r#type.is_identifier() {
                // Check & Parse Condition
                if let Some(condition) = self.peek() {
                    if condition.r#type.is_condition_op() {
                        return self.parse_condition();
                    }
                }
            }

            // Check & Parse Fn Call
            if token.r#type.is_fn_call() {
                return self.parse_function_call();
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

        Err(ParserError {
            message: format!("unexpected end of input while parsing expression"),
            token: None
        })
    }

    fn parse_token(&mut self) -> ParserResult<Node> {
        if let Some(token) = &self.current_token {
            if token.r#type.is_statement() {
                return self.parse_statement();
            }

            return self.parse_expr();
        }

        Err(ParserError {
            message: format!("Unhandled Token"),
            token: Some(self.current_token.clone().unwrap())
        })
    }

    pub fn parse(&mut self) -> ParserResult<Vec<Node>> {
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
