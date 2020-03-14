mod direction;
mod token;
mod program;

use token::{Token, char_to_token};
use program::{Program, StdinInputReader, NcursesInputReader};

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
    Ok(Program::new(parsed_contents, Box::new(NcursesInputReader::new())))
}

fn run_program(program: &mut Program) {
    while program.is_running() {
        program.step();
        print!("{}", program.get_last_output());
    }
}

struct DebugMSWindows {
    output_window: *mut i8,
    output_border_window: *mut i8,
    program_window: *mut i8,
    stack_window: *mut i8,
    last_output: String,
    cumulative_output: String,
}

impl DebugMSWindows {
    fn new() -> DebugMSWindows {
        let mut windows = DebugMSWindows {
            program_window: std::ptr::null_mut(),
            output_border_window: std::ptr::null_mut(),
            output_window: std::ptr::null_mut(),
            stack_window: std::ptr::null_mut(),
            last_output: String::new(),
            cumulative_output: String::new(),
        };

        windows._compute_window_geometry();
        windows
    }

    fn _compute_window_geometry(&mut self) {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(stdscr(), &mut max_y, &mut max_x);

        let right_pane_width = 13;
        let border_bottom = ((max_y as f32) * 0.8) as i32;
        let border_right  = max_x - right_pane_width;

        self.program_window = newwin(border_bottom, border_right, 0, 0);
        self.output_border_window = newwin(max_y - border_bottom, max_x, border_bottom, 0);
        self.output_window = newwin(max_y - border_bottom - 4, max_x - 2, border_bottom + 3, 1);
        self.stack_window = newwin(border_bottom, right_pane_width, 0, max_x - right_pane_width);

    }

    fn render_stack_window(&mut self, program: &Program) {
        wclear(self.stack_window);
        mvwaddstr(self.stack_window, 1, 1, "Stack:");

        for (index, element) in program.get_stack().iter().rev().enumerate() {
            mvwaddstr(self.stack_window, (2 + index) as i32, 1, &format!("{}", element));
        }

        box_(self.stack_window, 0, 0);
        wrefresh(self.stack_window);
    }

    fn render_program_window(&mut self, program: &Program) {
        for (y, line) in format!("{}", program).split("\n").enumerate() {
            let y = y as i32;

            if y - 1 == program.yptr() {
                for (x, c) in line.to_string().chars().enumerate() {
                    let x = x as i32;

                    if x == program.xptr() {
                        wattron(self.program_window, A_REVERSE());
                    }

                    mvwaddch(self.program_window, y, x + 1, c as u32);

                    if x == program.xptr() {
                        wattroff(self.program_window, A_REVERSE());
                    }
                }
            } else {
                mvwaddstr(self.program_window, y, 1, line);
            }
        }

        box_(self.program_window, 0, 0);
        wrefresh(self.program_window);
    }

    fn _render_cumulative_output(&mut self) {
        mvwaddstr(self.output_window, 0, 0, &self.cumulative_output);
        wrefresh(self.output_window);
    }

    fn render_output_window(&mut self) {
        scrollok(self.output_window, true);

        mvwaddstr(self.output_border_window, 1, 1, &format!("Last Output: {}", self.last_output));
        mvwaddstr(self.output_border_window, 2, 1, "Cumulative Output:");

        box_(self.output_border_window, 0, 0);
        wrefresh(self.output_border_window);

        self._render_cumulative_output();
    }

    fn render_ended_program_window(&mut self) {
        box_(self.output_border_window, 0, 0);
        mvwaddstr(self.output_border_window, 1, 1, &format!("{:<80}", "Program has ended"));
        wrefresh(self.output_border_window);

        self._render_cumulative_output();
    }

    fn log_output(&mut self, output: String) {
        self.last_output = output;
        self.cumulative_output += &self.last_output;
    }

    fn render(&mut self, program: &Program) {
        self.render_program_window(program);
        self.render_output_window();
        self.render_stack_window(program);
    }
}

fn debug_program(program: &mut Program) {
    initscr();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    let mut windows = DebugMSWindows::new();

    while program.is_running() {
        clear();
        refresh();

        windows.log_output(program.get_last_output());
        windows.render(program);

        program.step();
        noecho();
    }

    windows.render_ended_program_window();
    noecho();
    endwin();
}

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
