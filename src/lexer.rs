#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
    Command,
    StringLiteral,
    IntLiteral
}

#[derive(Debug, Clone)]
pub struct Token {
    pub r#type: TokenKind,
    pub value: Option<String>
}

pub fn lex(source: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = source.chars().peekable();

    while let Some(char) = chars.next() {
        // Check if string literal
        if char == '"' {
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

            tokens.push(Token { r#type: TokenKind::StringLiteral, value: Some(buffer) });
            continue;
        }

        // Check if int literal
        if char.is_numeric() {
            let mut buffer = String::new();

            buffer.push(char);

            while let Some(curr_char) = chars.peek() {
                if curr_char.is_numeric() {
                    buffer.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            tokens.push(Token { r#type: TokenKind::IntLiteral, value: Some(buffer) });
            continue;
        }

        // Check if its alphanumeric but doesnt start as number
        if char.is_alphanumeric() && !char.is_numeric() {
            let mut buffer = String::new();

            buffer.push(char);

            while let Some(curr_char) = chars.peek() {
                if curr_char.is_alphanumeric() {
                    buffer.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            // Check if its a command or an identifier
            match buffer.as_str() {
                "log" => { tokens.push(Token { r#type: TokenKind::Command, value: Some(buffer) }); },
                "logl" => { tokens.push(Token { r#type: TokenKind::Command, value: Some(buffer) }); },
                "set" => { tokens.push(Token { r#type: TokenKind::Command, value: Some(buffer) }); },
                _ => { tokens.push(Token { r#type: TokenKind::Identifier, value: Some(buffer) }) }
            }

            continue;
        }
    }

    tokens.to_owned()
}
