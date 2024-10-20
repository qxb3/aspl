use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
    Command,
    StringLiteral,
    IntLiteral,
    Boolean
}

#[derive(Debug, Clone)]
pub struct Token {
    pub r#type: TokenKind,
    pub value: Option<String>
}

pub struct Lexer {
    tokens: Vec<Token>
}

impl Lexer {
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    fn lex_string_lit(&mut self, chars: &mut Peekable<Chars>) {
        let mut buffer = String::new();

        while let Some(curr_char) = chars.peek() {
            if curr_char != &'"' {
                buffer.push(chars.next().unwrap());
            } else {
                break;
            }
        }

        // Ignore the current '"'
        chars.next();

        self.tokens.push(Token { r#type: TokenKind::StringLiteral, value: Some(buffer) });
    }

    fn lex_int_lit(&mut self, char: char, chars: &mut Peekable<Chars>) {
        let mut buffer = String::new();

        buffer.push(char);

        while let Some(curr_char) = chars.peek() {
            if curr_char.is_numeric() {
                buffer.push(chars.next().unwrap());
            } else {
                break;
            }
        }

        self.tokens.push(Token { r#type: TokenKind::IntLiteral, value: Some(buffer) });
    }

    fn lex_command(&mut self, char: char, chars: &mut Peekable<Chars>) {
        let mut buffer = String::new();

        buffer.push(char);

        while let Some(curr_char) = chars.peek() {
            if curr_char.is_alphanumeric() {
                buffer.push(chars.next().unwrap());
            } else {
                break;
            }
        }

        match buffer.as_str() {
            "log" | "logl" | "set"      => { self.tokens.push(Token { r#type: TokenKind::Command, value: Some(buffer) }); },
            "true" | "false"            => { self.tokens.push(Token { r#type: TokenKind::Boolean, value: Some(buffer) }); },
            _                           => { self.tokens.push(Token { r#type: TokenKind::Identifier, value: Some(buffer) }) }
        }
    }

    pub fn lex(&mut self, source: &str) -> Vec<Token> {
        let mut chars = source.chars().peekable();

        while let Some(char) = chars.next() {
            // Check if string literal
            if char == '"' {
                self.lex_string_lit(&mut chars);
                continue;
            }

            // Check if int literal
            if char.is_numeric() {
                self.lex_int_lit(char, &mut chars);
                continue;
            }

            // Check if its command / identifier / boolean
            if char.is_alphanumeric() && !char.is_numeric() {
                self.lex_command(char, &mut chars);
                continue;
            }
        }

        self.tokens.to_owned()
    }
}
