mod lexer;
mod parser;
mod interpreter;

use std::{env, fs, process::exit};
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

    let mut lexer = Lexer::new();
    let mut parser = Parser::new();

    let tokens = lexer.lex(source.as_str());
    let ast = parser.parse(tokens);
    println!("{:#?}", ast);

    // match interpreter::run(ast) {
    //     Ok(()) => (),
    //     Err(err) => {
    //         println!("{err}");
    //     }
    // }
}
