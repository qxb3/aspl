#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenTypes {
    Identifier,
    Command,
    StringLiteral,
    IntLiteral,
    BooleanLiteral,
    EqEq,
    NotEq,
    GThan,
    GThanEq,
    LThan,
    LThanEq,
    AND,
    OR,
    OpenCurly,
    CloseCurly,
}

impl TokenTypes {
    pub fn is_command(&self) -> bool { matches!(self, TokenTypes::Command) }
    pub fn is_identifier(&self) -> bool { matches!(self, TokenTypes::Identifier) }
    pub fn is_open_curly(&self) -> bool { matches!(self, TokenTypes::OpenCurly) }
    pub fn is_close_curly(&self) -> bool { matches!(self, TokenTypes::CloseCurly) }

    pub fn is_literal(&self) -> bool{
        return matches!(self,
            TokenTypes::IntLiteral |
            TokenTypes::StringLiteral |
            TokenTypes::BooleanLiteral);
    }

    pub fn is_condition_op(&self) -> bool {
        return matches!(self,
            TokenTypes::EqEq |
            TokenTypes::NotEq |
            TokenTypes::GThan |
            TokenTypes::GThanEq |
            TokenTypes::LThan |
            TokenTypes::LThanEq);
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub r#type: TokenTypes,
    pub value: Option<String>,
    pub col: usize,
    pub line: usize
}

#[derive(Debug)]
pub struct Lexer<T: Iterator<Item = char> + Clone> {
    chars: T,
    current_char: Option<char>,
    line: usize,
    col: usize
}

#[derive(Debug)]
pub struct LexerError {
    pub message: String,
    pub char: Option<char>
}

type LexerResult<T> = Result<T, LexerError>;

impl<T: Iterator<Item = char> + Clone> Lexer<T> {
    pub fn new(mut chars: T) -> Self {
        let current_char = chars.next();

        Self {
            chars,
            current_char,
            line: 1,
            col: 1
        }
    }

    fn lex_str_lit(&mut self) -> LexerResult<Token> {
        let mut buffer = String::new();

        // Ignore "
        self.advance();

        while let Some(char) = &self.current_char {
            if char.eq(&'"') {
                self.advance();
                break;
            }

            buffer.push(char.to_owned());
            self.advance();
        }

        let str_lit = Ok(Token {
            r#type: TokenTypes::StringLiteral,
            value: Some(buffer.to_owned()),
            line: self.line,
            col: self.col
        });

        self.col += buffer.len() + 2;
        str_lit
    }

    fn lex_int_lit(&mut self) -> LexerResult<Token> {
        let mut buffer = String::new();

        while let Some(char) = &self.current_char {
            if !char.is_numeric() {
                break;
            }

            buffer.push(char.to_owned());
            self.advance();
        }

        let int_lit = Ok(Token {
            r#type: TokenTypes::IntLiteral,
            value: Some(buffer.to_owned()),
            line: self.line,
            col: self.col
        });

        self.col += buffer.len();
        int_lit
    }

    fn lex_identifier(&mut self) -> LexerResult<Token> {
        let mut buffer = String::new();

        while let Some(char) = &self.current_char {
            if !char.is_alphanumeric() {
                break;
            }

            buffer.push(char.to_owned());
            self.advance();
        }

        let identifier = match buffer.as_str() {
            "log" | "logl"  |
            "set" | "check" |
            "while" => Token {
                r#type: TokenTypes::Command,
                value: Some(buffer.to_owned()),
                line: self.line,
                col: self.col
            },
            "true" | "false" => Token {
                r#type: TokenTypes::BooleanLiteral,
                value: Some(buffer.to_owned()),
                line: self.line,
                col: self.col
            },
            _ => Token {
                r#type: TokenTypes::Identifier,
                value: Some(buffer.to_owned()),
                line: self.line,
                col: self.col
            }
        };

        self.col += buffer.len();
        Ok(identifier)
    }

    fn parse_symbol(&mut self, char: char) -> LexerResult<Token> {
        let token_type = match char {
            '=' if self.peek().unwrap_or_default() == '=' => Some(TokenTypes::EqEq),
            '!' if self.peek().unwrap_or_default() == '=' => Some(TokenTypes::NotEq),
            '>' if self.peek().unwrap_or_default() == '=' => Some(TokenTypes::GThanEq),
            '<' if self.peek().unwrap_or_default() == '=' => Some(TokenTypes::LThanEq),

            '>' => Some(TokenTypes::GThan),
            '<' => Some(TokenTypes::LThan),

            '&' if self.peek().unwrap_or_default() == '&' => Some(TokenTypes::AND),
            '|' if self.peek().unwrap_or_default() == '|' => Some(TokenTypes::OR),

            '{' => Some(TokenTypes::OpenCurly),
            '}' => Some(TokenTypes::CloseCurly),

            _ => None,
        };

        if let Some(token_type) = token_type {
            if matches!(token_type,
                TokenTypes::EqEq |
                TokenTypes::NotEq |
                TokenTypes::GThanEq |
                TokenTypes::LThanEq |
                TokenTypes::AND |
                TokenTypes::OR) {
                self.advance();
            }

            self.advance();

            return Ok(Token {
                r#type: token_type,
                value: None,
                line: self.line,
                col: self.col
            });
        }

        Err(LexerError {
            message: "Unexpected end of input while lexing symbol".to_string(),
            char: None
        })
    }

    pub fn lex(&mut self) -> LexerResult<Vec<Token>> {
        let mut parsed_tokens: Vec<Token> = vec![];

        while let Some(char) = self.current_char {
            if char == '\n' {
                self.line += 1;
                self.col = 1;
                self.advance();

                continue;
            }

            if char == '"' {
                let str_lit = self.lex_str_lit()?;
                parsed_tokens.push(str_lit);

                continue;
            }

            if char.is_numeric() {
                let str_int = self.lex_int_lit()?;
                parsed_tokens.push(str_int);

                continue;
            }

            if char.is_alphanumeric() && !char.is_numeric() {
                let identifier = self.lex_identifier()?;
                parsed_tokens.push(identifier);

                continue;
            }

            if char.is_whitespace() {
                self.col += 1;
                self.advance();

                continue;
            }

            let symbol = self.parse_symbol(char)?;
            parsed_tokens.push(symbol);
        }

        Ok(parsed_tokens)
    }

    fn advance(&mut self) {
        self.current_char = self.chars.next();
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.clone().next()
    }
}
