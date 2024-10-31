mod lexer;
mod parser;
mod interpreter;

use std::{env, fs, process::exit};
use inline_colorization::*;
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

fn main() {
    let source_file = match env::args().nth(1) {
        Some(source_file) => source_file,
        None => {
            println!("Error: Specify the aspl file:");
            println!("$ aspl <input.aspl>");
            exit(1);
        }
    };

    let source = match fs::read_to_string(&source_file) {
        Ok(contents) => contents,
        Err(_) => {
            println!("Cannot read file: {source_file}");
            exit(1);
        }
    };

    let tokens = match Lexer::new(source.as_str().chars()).lex() {
        Ok(tokens) => tokens,
        Err(err) => {
            println!("{color_red}[ERROR]{color_reset} -> Lexing Error: {}.", err.message);

            if let Some(char) = err.char {
                println!("{color_yellow}[CHAR]{color_reset}  -> {:#?}.", char);
            }

            exit(1);
        }
    };

    // println!("{:#?}", tokens);

    let ast = match Parser::new(tokens.iter().cloned().into_iter()).parse() {
        Ok(ast) => ast,
        Err(err) => {
            println!("{color_red}[ERROR]{color_reset} -> Parsing Error: {}.", err.message);

            if let Some(token) = err.token {
                println!("{color_yellow}[POSITION]{color_reset} -> {}:{}", token.line, token.col);
                println!("{color_green}[TOKEN]{color_reset} -> {:#?}.", token);
            }

            exit(1);
        }
    };

    // println!("{:#?}", ast);

    let mut interpreter = Interpreter::new();
    if let Err(err) = interpreter.run(&ast) {
        println!("{color_red}[ERROR]{color_reset} -> {:?}: {}.", err.r#type, err.message);
        exit(1);
    }
}
