use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenTypes {
    Identifier,
    Command,
    StringLiteral,
    IntLiteral,
    Boolean,
    EqEq,
    NotEq,
    GThan,
    GThanEq,
    LThan,
    LThanEq,
    OpenCurly,
    CloseCurly
}

#[derive(Debug, Clone)]
pub struct Token {
    pub r#type: TokenTypes,
    pub value: Option<String>,
    pub col: usize,
    pub line: usize
}

pub struct Lexer {
    tokens: Vec<Token>
}

impl Lexer {
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    fn lex_string_lit(&mut self, chars: &mut Peekable<Chars>, line: &mut usize, col: &mut usize) {
        let mut buffer = String::new();

        while let Some(curr_char) = chars.peek() {
            if curr_char != &'"' {
                buffer.push(chars.next().unwrap());
                *col += 1;
            } else {
                break;
            }
        }

        // Ignore the current '"'
        chars.next();
        *col += 1;

        self.tokens.push(Token { r#type: TokenTypes::StringLiteral, value: Some(buffer), line: line.to_owned(), col: col.to_owned() });
    }

    fn lex_int_lit(&mut self, char: char, chars: &mut Peekable<Chars>, line: &mut usize, col: &mut usize) {
        let mut buffer = String::new();

        buffer.push(char);

        while let Some(curr_char) = chars.peek() {
            if curr_char.is_numeric() {
                buffer.push(chars.next().unwrap());
                *col += 1;
            } else {
                break;
            }
        }

        self.tokens.push(Token { r#type: TokenTypes::IntLiteral, value: Some(buffer), line: line.to_owned(), col: col.to_owned() });
    }

    fn lex_command(&mut self, char: char, chars: &mut Peekable<Chars>, line: &mut usize, col: &mut usize) {
        let mut buffer = String::new();

        buffer.push(char);

        while let Some(curr_char) = chars.peek() {
            if curr_char.is_alphanumeric() {
                buffer.push(chars.next().unwrap());
            } else {
                break;
            }
        }

        *col += buffer.len();

        match buffer.as_str() {
            "log" | "logl" | "set" | "check"    => { self.tokens.push(Token { r#type: TokenTypes::Command, value: Some(buffer), line: line.to_owned(), col: col.to_owned() }); },
            "true" | "false"                    => { self.tokens.push(Token { r#type: TokenTypes::Boolean, value: Some(buffer), line: line.to_owned(), col: col.to_owned() }); },
            _                                   => { self.tokens.push(Token { r#type: TokenTypes::Identifier, value: Some(buffer), line: line.to_owned(), col: col.to_owned() }); }
        }
    }

    pub fn lex(&mut self, source: &str) -> Vec<Token> {
        let mut chars = source.chars().peekable();
        let mut line: usize = 1;
        let mut col: usize = 1;

        while let Some(char) = chars.next() {
            // Check if string literal
            if char == '"' {
                self.lex_string_lit(&mut chars, &mut line, &mut col);
                continue;
            }

            // Check if int literal
            if char.is_numeric() {
                self.lex_int_lit(char, &mut chars, &mut line, &mut col);
                continue;
            }

            // Check if its command / identifier / boolean
            if char.is_alphanumeric() && !char.is_numeric() {
                self.lex_command(char, &mut chars, &mut line, &mut col);
                continue;
            }

            match char {
                // Check if comparison operators
                comp if comp == '=' && chars.peek().unwrap() == &'=' => { self.tokens.push(Token { r#type: TokenTypes::EqEq, value: None, line, col }); },
                comp if comp == '!' && chars.peek().unwrap() == &'=' => { self.tokens.push(Token { r#type: TokenTypes::EqEq, value: None, line, col }); },
                comp if comp == '>' && chars.peek().unwrap() == &'=' => { self.tokens.push(Token { r#type: TokenTypes::GThanEq, value: None, line, col }); },
                comp if comp == '<' && chars.peek().unwrap() == &'=' => { self.tokens.push(Token { r#type: TokenTypes::LThanEq, value: None, line, col }); },
                '>' => { self.tokens.push(Token { r#type: TokenTypes::GThan, value: None, line, col }); },
                '<' => { self.tokens.push(Token { r#type: TokenTypes::LThan, value: None, line, col }); }

                '{' => { self.tokens.push(Token { r#type: TokenTypes::OpenCurly, value: None, line, col }); },
                '}' => { self.tokens.push(Token { r#type: TokenTypes::CloseCurly, value: None, line, col }); },

                _ => ()
            }

            if char == '\n' {
                line += 1;
                col = 1;
            }
        }

        self.tokens.to_owned()
    }
}
