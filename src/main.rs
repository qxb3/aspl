mod lexer;
mod parser;
// mod interpreter;

use std::{env, fs, process::exit};
use inline_colorization::*;
// use interpreter::Interpreter;
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

    let tokens = Lexer::new().lex(source.as_str());
    let ast = match Parser::new(tokens.iter().cloned().into_iter()).parse() {
        Ok(ast) => ast,
        Err(err) => {
            println!("{color_red}[ERROR]{color_reset} -> {err}.");
            exit(1);
        }
    };

    println!("{:#?}", ast);

    // let interpreter = Interpreter::new();
    // match interpreter.run(ast) {
    //     Ok(()) => (),
    //     Err(err) => {
    //         println!("{err}");
    //     }
    // }
}
