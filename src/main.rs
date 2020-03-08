mod direction;
mod token;
mod program;

use token::{Token, char_to_token};
use program::Program;

#[macro_use]
extern crate lazy_static;

use ncurses::*;

use std::fs;
use std::io;
use clap::{App, Arg};

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
        print!("{}", program.get_last_output());
    }
}

fn debug_program(program: &mut Program) {
    let mut cumulative_output = String::new();

    initscr();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    while program.is_running() {
        clear();
        refresh();

        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(stdscr(), &mut max_y, &mut max_x);

        let right_pane_width = 13;
        let border_bottom = ((max_y as f32) * 0.8) as i32;
        let border_right  = max_x - right_pane_width;

        let last_output = program.get_last_output();
        cumulative_output += &last_output;

        let program_window = newwin(border_bottom, border_right, 0, 0);

        for (y, line) in format!("{}", program).split("\n").enumerate() {
            let y = y as i32;

            if y - 1 == program.yptr() {
                for (x, c) in line.to_string().chars().enumerate() {
                    let x = (x as i32);

                    if x == program.xptr() {
                        wattron(program_window, A_REVERSE());
                    }

                    mvwaddch(program_window, y, x + 1, c as u32);

                    if x == program.xptr() {
                        wattroff(program_window, A_REVERSE());
                    }
                }
            } else {
                mvwaddstr(program_window, y, 1, line);
            }
        }

        box_(program_window, 0, 0);
        wrefresh(program_window);
        
        let output_window_border = newwin(max_y - border_bottom, max_x, border_bottom, 0);
        let output_window = newwin(max_y - border_bottom - 4, max_x - 2, border_bottom + 3, 1);
        scrollok(output_window, true);

        mvwaddstr(output_window_border, 1, 1, &format!("Last Output: {}", last_output));
        mvwaddstr(output_window_border, 2, 1, "Cumulative Output:");
        mvwaddstr(output_window, 0, 0, &format!("{}", cumulative_output));

        box_(output_window_border, 0, 0);
        wrefresh(output_window_border);
        wrefresh(output_window);


        let stack_window = newwin(border_bottom, right_pane_width, 0, max_x - right_pane_width);

        mvwaddstr(stack_window, 1, 1, "Stack:");

        for (index, element) in program.get_stack().iter().rev().enumerate() {
            mvwaddstr(stack_window, (2 + index) as i32, 1, &format!("{}", element));
        }

        box_(stack_window, 0, 0);
        wrefresh(stack_window);

        program.step();
        getch();
    }

    addstr("\nProgram has ended\n");
    getch();
    endwin();
}

/*
|                                                                                                |          |
|                                                                                                |          |
|                                                                                                |          |
|                                                                                                |          |
|                                                                                                |          |
|                                                                                                |          |
|                                                                                                |          |
|                                                                                                |          |
|                                                                                                |          |
|                                                                                                |          |
|                                                                                                |          |
|                                                                                                |          |
|                                                                                                |          |
|                                                                                                |          |
|                                                                                                |          |
|                                                                                                |          |
|                                                                                                |          |
|                                                                                                |          |
|                                                                                                |          |
|                                                                                                |          |
|-----------------------------------------------------------------------------------------------------------|
|Last output: a                                                                                             |
|Output:                                                                                                    |
|                                                                                                           |
|                                                                                                           |
|                                                                                                           |
|                                                                                                           |
|                                                                                                           |
|                                                                                                           |
*/

fn main() {
    let matches = App::new("Rusty Fungus")
        .version("1.0")
        .author("Elijah Mirecki & Joshua Wolfe")
        .about("A befunge intepreter")
        .arg(Arg::with_name("debug")
             .short("d")
             .long("debug")
             .help("Runs the program in debug mode")
             .takes_value(false))
        .arg(Arg::with_name("INPUT")
             .help("Sets the Befunge program file to use")
             .required(true)
             .index(1))
        .get_matches();

    let filename = matches.value_of("INPUT").unwrap().to_string();

    match load_program(filename) {
        Ok(mut program) => {
            if matches.is_present("debug") {
                debug_program(&mut program);
            } else {
                run_program(&mut program);
            }
        },
        Err(e) => exit_with_message(&e.to_string()),
    };
}
