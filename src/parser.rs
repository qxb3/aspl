use crate::lexer::{Lexer, Token, TokenTypes};
use inline_colorization::*;
use std::{env, fs, mem::discriminant, path::{Path, PathBuf}};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Literals {
    String(String),
    Int(i64),
    Boolean(bool),
    Array(Vec<Literals>)
}

impl Literals {
    pub fn name(&self) -> &str {
        match self {
            Literals::Int(_) => "int",
            Literals::String(_) => "string",
            Literals::Boolean(_) => "boolean",
            Literals::Array(_) => "array"
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Node {
    Literal(Literals),
    Identifier(String),
    Return(Box<Node>),
    Break,
    Var {
        identifier: Box<Node>,
        value: Box<Node>
    },
    ArrayAccess {
        identifier: Box<Node>,
        index: Box<Node>
    },
    Condition {
        left: Box<Node>,
        condition: String,
        right: Box<Node>
    },
    MathExpr {
        left: Box<Node>,
        op: String,
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
    Source {
        file_name: String,
        cwd: PathBuf,
        ast: Vec<Node>
    },

    // Statements
    Log {
        r#type: String,
        args: Vec<Box<Node>>
    },
    Update {
        identifier: Box<Node>,
        value: Box<Node>
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

#[derive(Debug)]
pub struct ParserError {
    pub message: String,
    pub token: Option<Token>,
}

type ParserResult<T> = Result<T, ParserError>;

#[derive(Debug, Clone)]
pub struct Parser<T: Iterator<Item = Token> + Clone> {
    tokens: T,
    current_token: Option<Token>
}

impl<T: Iterator<Item = Token> + Clone> Parser<T> {
    pub fn new(mut tokens: T) -> Self {
        let current_token = tokens.next();

        Self {
            tokens,
            current_token
        }
    }

    fn parse_set_statement(&mut self) -> ParserResult<Node> {
        self.advance();

        let identifier = self.parse_identifier()?;

        let value = match &self.current_token.clone() {
            Some(node) => match node {
                node if node.r#type.is_literal() ||
                        node.r#type.is_open_bracket()   => self.parse_literal()?,
                node if node.r#type.is_fn_call()        => self.parse_function_call()?,
                node if node.r#type.is_identifier() &&
                        self.peek().is_some() &&
                        self.peek().unwrap()
                            .r#type.is_open_bracket()   => self.parse_array_access()?,
                node if node.r#type.is_identifier()     => self.parse_identifier()?,
                _ => {
                    return Err(ParserError {
                        message: format!(
                            "Expected a literal/identifier/function call, but found {:?}",
                            node.r#type
                        ),
                        token: Some(node.clone()),
                    })
                }
            },
            None => {
                return Err(ParserError {
                    message: format!("Unexpected end of input while parsing set statement"),
                    token: None,
                })
            }
        };

        Ok(Node::Var {
            identifier: Box::new(identifier),
            value: Box::new(value),
        })
    }

    fn parse_update_statement(&mut self) -> ParserResult<Node> {
        self.advance();

        let identifier = self.parse_identifier()?;

        let value = match &self.current_token.clone() {
            Some(node) => match node {
                node if node.r#type.is_literal() ||
                        node.r#type.is_open_bracket()   => self.parse_literal()?,
                node if node.r#type.is_identifier()     => self.parse_identifier()?,
                node if node.r#type.is_fn_call()        => self.parse_function_call()?,
                _ => {
                    return Err(ParserError {
                        message: format!(
                            "Expected a literal/identifier/function call, but found {:?}",
                            node.r#type
                        ),
                        token: Some(node.clone()),
                    })
                }
            },
            None => {
                return Err(ParserError {
                    message: format!("Unexpected end of input while parsing update statement"),
                    token: None,
                })
            }
        };

        Ok(Node::Update {
            identifier: Box::new(identifier),
            value: Box::new(value),
        })
    }

    fn parse_log_statement(&mut self, statement: String) -> ParserResult<Node> {
        self.advance();

        let mut args: Vec<Box<Node>> = vec![];

        while let Some(arg) = &self.current_token {
            match arg.r#type {
                arg if arg.is_literal() ||
                        arg.is_open_bracket()           => args.push(Box::new(self.parse_literal()?)),
                arg if arg.is_identifier() &&
                        self.peek().is_some() &&
                        self.peek().unwrap()
                            .r#type.is_open_bracket()   => args.push(Box::new(self.parse_array_access()?)),
                arg if arg.is_identifier()              => args.push(Box::new(self.parse_identifier()?)),
                arg if arg.is_fn_call()                 => args.push(Box::new(self.parse_function_call()?)),
                _ => break,
            }
        }

        if args.is_empty() {
            return Err(ParserError {
                message: format!(
                    "Unexpected end of input while parsing {} statement",
                    statement
                ),
                token: None,
            });
        }

        Ok(Node::Log {
            r#type: statement,
            args,
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
                            scope: Box::new(scope),
                        });
                    }
                }
            }

            if token.r#type.is_literal() {
                let literal = self.parse_literal()?;
                let scope = self.parse_scope()?;

                return Ok(Node::Check {
                    condition: Box::new(literal),
                    scope: Box::new(scope),
                });
            }
        }

        Err(ParserError {
            message: format!("Unexpected end of input while parsing check statement"),
            token: None,
        })
    }

    fn parse_while_statement(&mut self) -> ParserResult<Node> {
        self.advance();

        if let Some(token) = &self.current_token {
            if token.r#type.is_literal() || token.r#type.is_identifier() {
                if let Some(condition) = self.peek() {
                    if condition.r#type.is_condition_op() || condition.r#type.is_open_bracket() {
                        let condition = self.parse_condition()?;
                        let scope = self.parse_scope()?;

                        return Ok(Node::While {
                            condition: Box::new(condition),
                            scope: Box::new(scope),
                        });
                    }
                }
            }

            if token.r#type.is_literal() || token.r#type.is_open_bracket() {
                let literal = self.parse_literal()?;
                let scope = self.parse_scope()?;

                return Ok(Node::While {
                    condition: Box::new(literal),
                    scope: Box::new(scope),
                });
            }

            return Err(ParserError {
                message: format!("Expected a condition or literal, but found {:?}", token),
                token: Some(token.clone()),
            });
        }

        Err(ParserError {
            message: format!("Unexpected end of input while parsing while statement"),
            token: None,
        })
    }

    fn parse_break(&mut self) -> ParserResult<Node> {
        self.advance();

        Ok(Node::Break)
    }

    fn parse_function(&mut self) -> ParserResult<Node> {
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
            scope: Box::new(scope),
        })
    }

    fn parse_return(&mut self) -> ParserResult<Node> {
        self.advance();

        if let Some(token) = &self.current_token {
            if token.r#type.is_literal() || token.r#type.is_open_bracket() {
                let ret_identifier = self.parse_literal()?;
                return Ok(Node::Return(Box::new(ret_identifier)));
            }

            if token.r#type.is_identifier() {
                let ret_literal = self.parse_identifier()?;
                return Ok(Node::Return(Box::new(ret_literal)));
            }

            if token.r#type.is_fn_call() {
                let ret_fn_call = self.parse_function_call()?;
                return Ok(Node::Return(Box::new(ret_fn_call)));
            }

            return Err(ParserError {
                message: format!(
                    "Expected a literal/identifier/fn_call, but found {:?}",
                    token.r#type
                ),
                token: Some(token.clone()),
            });
        }

        Err(ParserError {
            message: format!("Unexpected end of input while parsing return"),
            token: None,
        })
    }

    // Parse all statements
    fn parse_statement(&mut self) -> ParserResult<Node> {
        if let Some(token) = &self.current_token {
            let statement = token.value.clone().unwrap();

            match statement.as_str() {
                "set"           => return self.parse_set_statement(),
                "update"        => return self.parse_update_statement(),
                "log" | "logl"  => return self.parse_log_statement(statement),
                "check"         => return self.parse_check_statement(),
                "while"         => return self.parse_while_statement(),
                "fn"            => return self.parse_function(),
                "ret"           => return self.parse_return(),
                "break"         => return self.parse_break(),

                _ => {
                    return Err(ParserError {
                        message: format!("Expected a statement, but found {:?}", token.r#type),
                        token: Some(token.clone()),
                    })
                }
            }
        }

        Err(ParserError {
            message: format!("Unexpected end of input while parsing statement"),
            token: None,
        })
    }

    fn parse_array_literal(&mut self) -> ParserResult<Literals> {
        self.advance();

        let mut values: Vec<Literals> = vec![];

        while let Some(token) = &self.current_token {
            if token.r#type.is_close_bracket() {
                self.advance();
                break;
            }

            let value: Literals = match token.r#type {
                TokenTypes::IntLiteral      => Literals::Int(token.value.clone().unwrap().parse().unwrap()),
                TokenTypes::StringLiteral   => Literals::String(token.value.clone().unwrap().parse().unwrap()),
                TokenTypes::BooleanLiteral  => Literals::Boolean(token.value.clone().unwrap().parse().unwrap()),
                TokenTypes::OpenBracket     => self.parse_array_literal()?,
                _ => {
                    return Err(ParserError {
                        message: format!(
                            "Expected a literal, but found {:?}",
                            token.r#type
                        ),
                        token: Some(self.current_token.clone().unwrap()),
                    })
                }
            };

            if matches!(value,
                Literals::Int(_) |
                Literals::String(_) |
                Literals::Boolean(_)) {
                self.advance();
            }

            values.push(value);
        }

        if !values.iter().all(|value| discriminant(value) == discriminant(&values[0])) {
            return Err(ParserError {
                message: format!("Cannot have two or more types in array"),
                token: Some(self.current_token.clone().unwrap()),
            });
        }

        Ok(Literals::Array(values))
    }

    fn parse_literal(&mut self) -> ParserResult<Node> {
        if let Some(token) = &self.current_token.clone() {
            let value: Literals = match token.r#type {
                TokenTypes::IntLiteral      => Literals::Int(token.value.clone().unwrap().parse().unwrap()),
                TokenTypes::StringLiteral   => Literals::String(token.value.clone().unwrap().parse().unwrap()),
                TokenTypes::BooleanLiteral  => Literals::Boolean(token.value.clone().unwrap().parse().unwrap()),
                TokenTypes::OpenBracket     => self.parse_array_literal()?,
                _ => return Err(ParserError {
                    message: format!("Expected a literal, but found {:?}", token.r#type),
                    token: Some(token.clone()),
                })
            };

            if matches!(value,
                Literals::Int(_) |
                Literals::String(_) |
                Literals::Boolean(_)) {
                self.advance();
            }

            return Ok(Node::Literal(value));
        }

        Err(ParserError {
            message: format!("Unexpected end of input while parsing literal"),
            token: None,
        })
    }

    fn parse_identifier(&mut self) -> ParserResult<Node> {
        if let Some(token) = &self.current_token.clone() {
            if !token.r#type.is_identifier() {
                return Err(ParserError {
                    message: format!("Expected a identifier, but found {:?}", token.r#type),
                    token: Some(token.clone()),
                });
            }

            self.advance();

            return Ok(Node::Identifier(token.value.clone().unwrap()));
        }

        Err(ParserError {
            message: format!("Unexpected end of input while parsing identifier"),
            token: None,
        })
    }

    fn parse_array_access(&mut self) -> ParserResult<Node> {
        let mut current_identifier = self.parse_identifier()?;

        while let Some(token) = &self.current_token {
            if !token.r#type.is_open_bracket() {
                break;
            }

            self.advance();

            let index = match &self.current_token {
                Some(token) if token.r#type.is_literal()        => self.parse_literal()?,
                Some(token) if token.r#type.is_identifier()     => self.parse_identifier()?,
                Some(token) => return Err(ParserError {
                    message: format!("Expected an index, but found {:?}", token.r#type),
                    token: None,
                }),
                None => return Err(ParserError {
                    message: "Unexpected end of input while parsing array access".to_string(),
                    token: None,
                })
            };

            if let Some(token) = &self.current_token {
                if !token.r#type.is_close_bracket() {
                    return Err(ParserError {
                        message: format!("Expected close bracket, but found: {:?}", token.r#type),
                        token: Some(token.clone()),
                    });
                }

                self.advance();
            } else {
                return Err(ParserError {
                    message: "Unexpected end of input while parsing array access".to_string(),
                    token: None,
                });
            }

            current_identifier = Node::ArrayAccess {
                identifier: Box::new(current_identifier),
                index: Box::new(index),
            };
        }

        Ok(current_identifier)
    }

    fn parse_condition(&mut self) -> ParserResult<Node> {
        let left = match &self.current_token {
            Some(left) => match left {
                node if node.r#type.is_identifier() &&
                        self.peek().is_some() &&
                        self.peek().unwrap()
                            .r#type.is_open_bracket()   => self.parse_array_access()?,
                left if left.r#type.is_identifier()     => self.parse_identifier()?,
                left if left.r#type.is_literal()        => self.parse_literal()?,
                left => {
                    return Err(ParserError {
                        message: format!("Expected a identifier or literal, but found {:?}", left),
                        token: Some(left.clone()),
                    })
                }
            },
            None => {
                return Err(ParserError {
                    message: format!("Unexpected end of input while parsing condition"),
                    token: None,
                })
            }
        };

        let condition = match &self.current_token {
            Some(token) => match token.r#type {
                TokenTypes::EqEq    => "==",
                TokenTypes::NotEq   => "!=",
                TokenTypes::GThan   => ">",
                TokenTypes::GThanEq => ">=",
                TokenTypes::LThan   => "<",
                TokenTypes::LThanEq => "<=",
                token_type => {
                    return Err(ParserError {
                        message: format!("Expected a condition, but found {:?}", token_type),
                        token: Some(token.clone()),
                    })
                }
            },
            None => {
                return Err(ParserError {
                    message: format!("Unexpected end of input while parsing condition"),
                    token: None,
                })
            }
        };

        self.advance();

        let right = match &self.current_token {
            Some(right) => match right {
                node if node.r#type.is_identifier() &&
                        self.peek().is_some() &&
                        self.peek().unwrap()
                            .r#type.is_open_bracket()   => self.parse_array_access()?,
                right if right.r#type.is_identifier()   => self.parse_identifier()?,
                right if right.r#type.is_literal()      => self.parse_literal()?,
                right => {
                    return Err(ParserError {
                        message: format!("Expected a identifier or literal, but found {:?}", right),
                        token: Some(right.clone()),
                    })
                }
            },
            None => {
                return Err(ParserError {
                    message: format!("Unexpected end of input while parsing condition"),
                    token: None,
                })
            }
        };

        Ok(Node::Condition {
            left: Box::new(left),
            condition: condition.to_string(),
            right: Box::new(right),
        })
    }

    fn parse_math_expr(&mut self) -> ParserResult<Node> {
        self.advance();

        let mut stack: Vec<Token> = vec![];
        let mut tokens: Vec<Token> = vec![];

        let token = match &self.current_token {
            Some(token) if token.r#type.is_open_paren() => token.clone(),
            Some(token) => return Err(ParserError {
                message: format!("Expected an open parenthesis on @math, but found: {:?}", token.r#type),
                token: Some(token.clone()),
            }),
            None => return Err(ParserError {
                message: "Unexpected end of input, expected '('".to_string(),
                token: None,
            }),
        };

        stack.push(token);
        self.advance();

        while let Some(token) = &self.current_token {
            if token.r#type.is_close_paren() && stack.len() == 1 {
                stack.pop();
                self.advance();
                break;
            }

            if token.r#type.is_close_paren() {
                if stack.is_empty() || !stack.last().unwrap().r#type.is_open_paren() {
                    return Err(ParserError {
                        message: "Mismatched parentheses".to_string(),
                        token: Some(token.clone()),
                    });
                }
                stack.pop();
                tokens.push(token.clone());
                self.advance();

                continue;
            }

            match token {
                token if token.r#type.is_math_op() ||
                        token.r#type.is_literal()  ||
                        token.r#type.is_identifier() => tokens.push(token.clone()),

                token if token.r#type.is_open_paren() => {
                    stack.push(token.clone());
                    tokens.push(token.clone());
                }

                _ => return Err(ParserError {
                    message: format!("Unexpected token in math expression: {:?}", token.r#type),
                    token: Some(token.clone()),
                }),
            }

            self.advance();
        }

        if !stack.is_empty() {
            return Err(ParserError {
                message: "Unmatched open parenthesis".to_string(),
                token: stack.last().cloned(),
            });
        }

        Ok(self.math_parse(tokens)?)
    }

    fn math_parse(&mut self, tokens: Vec<Token>) -> ParserResult<Node> {
        let mut output_stack: Vec<Node> = vec![];
        let mut operator_stack: Vec<String> = vec![];

        let mut i = 0;

        while i < tokens.len() {
            let token = &tokens[i];

            match &token {
                token if token.r#type.is_literal() => {
                    output_stack.push(Node::Literal(Literals::Int(token.value.clone().unwrap().parse().unwrap())));
                },
                token if token.r#type.is_identifier() => {
                    output_stack.push(Node::Identifier(token.value.clone().unwrap()));
                }
                token if token.r#type.is_math_op() => {
                    let op = token.value.clone().unwrap();

                    while !operator_stack.is_empty() &&
                        self.math_precedence(&operator_stack.last().unwrap()) >= self.math_precedence(&op)
                    {
                        let top_op = operator_stack.last().unwrap();
                        if (top_op == "+" || top_op == "-") && (op == "*" || op == "/") {
                            break;
                        }

                        let operator = operator_stack.pop().unwrap();
                        let right = output_stack.pop().unwrap();
                        let left = output_stack.pop().unwrap();

                        output_stack.push(Node::MathExpr {
                            left: Box::new(left),
                            op: operator,
                            right: Box::new(right),
                        });
                    }
                    operator_stack.push(op);
                },

                token if token.r#type.is_open_paren() => {
                    operator_stack.push("(".to_string());
                },

                token if token.r#type.is_close_paren() => {
                    while operator_stack.last().unwrap() != "(" {
                        let operator = operator_stack.pop().unwrap();
                        let right = output_stack.pop().unwrap();
                        let left = output_stack.pop().unwrap();

                        output_stack.push(Node::MathExpr {
                            left: Box::new(left),
                            op: operator,
                            right: Box::new(right),
                        });
                    }
                    operator_stack.pop();
                },

                _ => {
                    return Err(ParserError {
                        message: format!("Unexpected token: {:?}", token.r#type),
                        token: Some(token.clone()),
                    });
                }
            }

            i += 1;
        }

        while !operator_stack.is_empty() {
            let operator = operator_stack.pop().unwrap();
            let right = output_stack.pop().unwrap();
            let left = output_stack.pop().unwrap();

            output_stack.push(Node::MathExpr {
                left: Box::new(left),
                op: operator,
                right: Box::new(right),
            });
        }

        if output_stack.len() != 1 {
            return Err(ParserError {
                message: "Unexpected number of nodes in output stack".to_string(),
                token: None,
            })
        }

        Ok(output_stack.pop().unwrap())
    }

    fn math_precedence(&self, op: &str) -> i64 {
        match op {
            "+" | "-" => 1,
            "*" | "/" => 2,
            _ => 0,
        }
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

        Ok(Node::Scope { body })
    }

    fn parse_function_call(&mut self) -> ParserResult<Node> {
        let identifier = match &self.current_token {
            Some(token) => {
                if let Some(fn_call_name) = &token.value {
                    if fn_call_name == "source" {
                        return self.parse_source();
                    }

                    if fn_call_name == "math" {
                        return self.parse_math_expr();
                    }
                }

                Node::Identifier(token.clone().value.unwrap_or_default())
            }
            None => {
                return Err(ParserError {
                    message: format!("Unexpected end of input while parsing function call"),
                    token: None,
                })
            }
        };

        self.advance();

        let mut args: Vec<Box<Node>> = vec![];

        while let Some(token) = &self.current_token {
            let foo = match token.r#type {
                r#type if r#type.is_literal() => self.parse_literal()?,
                r#type if r#type.is_identifier() => self.parse_identifier()?,
                _ => break,
            };

            args.push(Box::new(foo));
        }

        Ok(Node::FunctionCall {
            identifier: Box::new(identifier),
            args,
        })
    }

    // The only function that has comments because its kinda confusing
    // Works as cd. you cd to the current dir the source will go to
    // Basically thats it.
    // Idk why even this exists but yeah.
    fn parse_source(&mut self) -> ParserResult<Node> {
        self.advance();

        let cwd = match env::current_dir() {
            Ok(cwd) => cwd,
            Err(_) => return Err(ParserError {
                message: format!("Cannot get the current working directory"),
                token: None
            })
        };

        let source_path = match self.parse_literal() {
            Ok(Node::Literal(Literals::String(source_path))) => source_path,
            Err(err) => return Err(err),
            _ => unreachable!(),
        };

        let source_absolute_path = match Path::new(&cwd.join(&source_path)).canonicalize() {
            Ok(file_path) => file_path,
            Err(err) => return Err(ParserError {
                message: format!("Failed to parse file path {:?}: {source_path}", err.to_string()),
                token: None
            })
        };

        if let Err(_) = env::set_current_dir(&source_absolute_path.parent().unwrap()) {
            return Err(ParserError {
                message: format!("Failed to change env directory to: {:?}", &source_absolute_path),
                token: None
            });
        }

        let source = match fs::read_to_string(&source_absolute_path) {
            Ok(contents) => contents,
            Err(_) => {
                return Err(ParserError {
                    message: format!("Cannot find file {:?}", &source_absolute_path),
                    token: None,
                })
            }
        };

        let tokens = match Lexer::new(source.as_str().chars()).lex() {
            Ok(tokens) => tokens,
            Err(err) => {
                return Err(ParserError {
                    message: format!(
                        r#"
                        Error while lexing file: {:?}
                        {color_red}[ERROR]{color_reset} -> Lexing Error: {}.
                    "#,
                        source_absolute_path, err.message
                    ),
                    token: None,
                })
            }
        };

        let ast = Parser::new(tokens.iter().cloned().into_iter()).parse()?;

        Ok(Node::Source {
            file_name: source_path,
            cwd: cwd.clone(),
            ast
        })
    }

    // Parse all expressions
    fn parse_expr(&mut self) -> ParserResult<Node> {
        if let Some(token) = &self.current_token {
            // Check & Parse Array Access
            if token.r#type.is_identifier() {
                if let Some(condition) = self.peek() {
                    if condition.r#type.is_open_bracket() {
                        return self.parse_array_access();
                    }
                }
            }

            // Check & Parse Condition
            if token.r#type.is_literal() || token.r#type.is_identifier() {
                if let Some(condition) = self.peek() {
                    if condition.r#type.is_condition_op() {
                        return self.parse_condition();
                    }
                }
            }

            // Check & Parse Import/Fn_Call
            if token.r#type.is_fn_call() {
                return self.parse_function_call();
            }

            // Check & Parse Literal
            if token.r#type.is_literal() || token.r#type.is_open_bracket() {
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
            token: None,
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
            token: Some(self.current_token.clone().unwrap()),
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
