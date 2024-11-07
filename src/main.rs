mod lexer;
mod parser;
mod interpreter;

use std::{env, fs, path::{Path, PathBuf}, process::exit};
use inline_colorization::*;
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

fn main() {
    let mut args = env::args().skip(1);

    let (source_path, source_parent) = match args.next() {
        Some(arg) if !arg.ends_with(".aspl") => {
            println!("{color_red}[ERROR]{color_reset} -> Invalid file extension.");
            exit(1);
        },
        Some(arg) => {
            let source_parent = match PathBuf::from(&arg.clone()).parent() {
                Some(parent) => parent.to_path_buf(),
                None => {
                    println!("{color_red}[ERROR]{color_reset} -> Cannot get {} parent path.", arg);
                    exit(1);
                }
            };

            (PathBuf::from(arg), source_parent)
        },
        None => {
            println!("{color_red}[ERROR]{color_reset} -> Specify the aspl file:");
            println!("{color_green}[USAGE]{color_reset} -> $ aspl <input.aspl>");
            exit(1);
        }
    };

    let cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(err) => {
            println!("{color_green}[ERROR]{color_reset} -> Cannot get the current working directory.");
            println!("{color_green}[STACK]{color_reset} -> {:?}", err);
            exit(1);
        }
    };

    if let Err(_) = env::set_current_dir(&cwd.join(&source_parent)) {
        println!("Failed to change env directory to: {:?}", &cwd);
        exit(1);
    }

    let source = match fs::read_to_string(&cwd.join(&source_path)) {
        Ok(contents) => contents,
        Err(_) => {
            println!("{color_red}[ERROR]{color_reset} -> Cannot read file: {:?}", source_path);
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

    let mut interpreter = Interpreter::new(cwd.clone());
    if let Err(err) = interpreter.run(&ast) {
        println!("{color_red}[ERROR]{color_reset} -> {:?}: {}.", err.r#type, err.message);
        exit(1);
    }
}
