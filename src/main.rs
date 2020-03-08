mod direction;
mod token;
mod program;

use token::{Token, char_to_token};
use program::Program;

#[macro_use]
extern crate lazy_static;

use std::env;
use std::fs;
use std::io;

fn exit_with_message(message: &str) {
    eprintln!("{}", message);
    std::process::exit(1);
}

fn lines_to_token_matrix(lines: std::str::Lines) -> Vec<Vec<Token>> {
    lines.map(|line| {
        line.chars().map(|c| char_to_token(c)).collect()
    }).collect()
}

fn load_program(filename: String) -> Result<Program, io::Error> {
    let contents = fs::read_to_string(filename)?;
    let parsed_contents = lines_to_token_matrix(contents.lines());
    Ok(Program::new(parsed_contents))
}

fn run_program(program: &mut Program) {
    while program.is_running() {
        program.step();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    match args.get(1) {
        Some(raw_filename) => {
            let filename = raw_filename.to_string();

            match load_program(filename) {
                Ok(mut program) => run_program(&mut program),
                Err(e) => exit_with_message(&e.to_string()),
            }
        },
        _ => exit_with_message("Usage: rustyfunges <filename>"),
    }
}
