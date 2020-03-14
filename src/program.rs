use crate::token::{Token, token_to_char, char_to_token};
use crate::direction::Direction;

use std::fmt;
use std::char;
use std::io;
use ncurses::{wgetch, mvwgetch, getch, wmove, wrefresh, mvwaddstr, box_, newwin, stdscr, getmaxyx};

pub trait InputReader {
    fn read_char(&mut self) -> i32;
    fn read_int(&mut self) -> i32;
}

pub struct StdinInputReader {
    buffered_line: String,
    buffered_index: usize,
}

pub struct NcursesInputReader {
    input_window: *mut i8,
}

impl StdinInputReader {
    pub fn new() -> StdinInputReader {
        StdinInputReader {
            buffered_line: String::new(),
            buffered_index: 0,
        }
    }

    fn _read_buffered_line_if_empty(&mut self) {
        if self.buffered_index >= self.buffered_line.len() {
            self.buffered_index = 0;

            let stdin = io::stdin();

            match stdin.read_line(&mut self.buffered_line) {
                Ok(_)  => {},
                Err(_) => self.buffered_line = String::new(),
            };
        }
    }
}

impl NcursesInputReader {
    pub fn new() -> NcursesInputReader {
        NcursesInputReader {
            input_window: std::ptr::null_mut(),
        }
    }

    fn _init_input_popup(&mut self) {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(stdscr(), &mut max_y, &mut max_x);
        let center_x = max_x / 2;
        let center_y = max_y / 2;
        let input_width = 40;
        let input_height = 6;

        self.input_window = newwin(
            input_height, input_width,
            center_y - input_height / 2, center_x - input_width / 2);
    }

    fn _render_input_popup(&mut self, input_type: String) {
        self._init_input_popup();

        box_(self.input_window, 0, 0);
        mvwaddstr(self.input_window, 2, 8, &format!("Please input a {}", input_type));
        wrefresh(self.input_window);
    }
}

impl InputReader for NcursesInputReader {
    fn read_char(&mut self) -> i32 {
        self._render_input_popup("character".to_string());
        mvwgetch(self.input_window, 3, 9)
    }

    fn read_int(&mut self) -> i32 {
        self._render_input_popup("integer".to_string());

        let mut number = 0;
        let mut c = mvwgetch(self.input_window, 3, 9);

        let negative_multiplier = if c == ('-' as i32) {
            c = wgetch(self.input_window);
            -1
        } else {
            1
        };

        loop {
            if (c > 57) || (c < 48) {
                break;
            }
            
            number *= 10;
            number += (c - 48);

            c = wgetch(self.input_window);
        }

        negative_multiplier * number
    }
}

fn read_int_from_string(s: &String, offset: usize) -> String {
    s.chars()
        .enumerate()
        .skip(offset)
        .take_while(|(index, c)| c.is_digit(10) || (*c == '-' && *index == offset))
        .map(|(_, c)| c)
        .collect::<String>()
}

impl InputReader for StdinInputReader {
    fn read_char(&mut self) -> i32 {
        self._read_buffered_line_if_empty();
        let maybe_char = self.buffered_line.chars().nth(self.buffered_index);
        self.buffered_index += 1;

        match maybe_char {
            Some(c) => c as i32,
            None    => 0,
        }
    }

    fn read_int(&mut self) -> i32 {
        self._read_buffered_line_if_empty();
        let str_int = read_int_from_string(&self.buffered_line, self.buffered_index);
        self.buffered_index += str_int.len() + 1;

        match str_int.parse::<i32>() {
            Ok(value) => value,
            Err(_)    => 0,
        }
    }
}

fn i32_to_char(value: i32) -> char {
    char::from_u32(value as u32).unwrap()
}

fn increment_wrap(value: i32, max_value: i32) -> i32 {
    if value == max_value - 1 {
        0
    } else {
        value + 1
    }
}

fn decrement_wrap(value: i32, max_value: i32) -> i32 {
    if value == 0 {
        max_value - 1
    } else {
        value - 1
    }
}

pub struct Program {
    xptr: i32,
    yptr: i32,
    direction: Direction,
    grid: Vec<Vec<Token>>,
    stack: Vec<i32>,
    is_running: bool,
    string_mode: bool,
    width: i32,
    last_output: String,
    input_reader: Box<dyn InputReader>,
}

impl Program {
    pub fn new(parsed_contents: Vec<Vec<Token>>, input_reader: Box<dyn InputReader>) -> Program {
        let max_width = parsed_contents.iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0) as i32;

        Program {
            xptr: 0,
            yptr: 0,
            direction: Direction::Right,
            grid: parsed_contents,
            stack: vec![],
            is_running: true,
            string_mode: false,
            width: max_width,
            last_output: String::new(),
            input_reader: input_reader,
        }
    }

    fn stack_pop(&mut self) -> i32 {
        match self.stack.pop() {
            Some(value) => value,
            None        => 0
        }
    }

    fn stack_push(&mut self, value: i32) {
        self.stack.push(value);
    }

    fn stack_peek(&self) -> i32 {
        match self.stack.last() {
            Some(value) => *value,
            None        => 0,
        }
    }

    fn binary_stack_op_push<F>(&mut self, op: F) where F: Fn(i32, i32) -> i32 {
        let a = self.stack_pop();
        let b = self.stack_pop();
        self.stack.push(op(a, b))
    }

    pub fn height(&self) -> i32 {
        self.grid.len() as i32
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn step(&mut self) {
        self.last_output = String::new();

        let current_token = self.get_token(self.xptr, self.yptr).unwrap();
        if self.string_mode {
            self.perform_string_action(current_token);
        } else {
            self.perform_action(current_token);
        }
        self.move_program_pointer();
    }

    fn set_token(&mut self, x: i32, y: i32, token: Token) {
        if x < 0 || y < 0  { // TODO: Be fancy and add rows/columns to the top/left
            panic!("Tried setting a negative value on the grid");
        }

        if x > self.width() {
            self.width = x;
        }

        let x = x as usize;
        let y = y as usize;

        // Expand the height to allow inserting at coordinate y
        for _ in self.grid.len()..=y {
            self.grid.push(vec![]);
        }

        // Expand row with no-ops to make room for the new token
        for _ in self.grid[y].len()..=x {
            self.grid[y].push(Token::Noop);
        }

        self.grid[y][x] = token;
    }

    fn get_token(&self, x: i32, y: i32) -> Option<Token> {
        if x < 0 || y < 0 || x >= self.width() || y >= self.height() {
            return None;
        }

        let x = x as usize;
        let y = y as usize;

        Some(match self.grid.get(y) {
            Some(row) => match row.get(x) {
                Some(token) => token.clone(),
                None => Token::Noop,
            },
            // Default to Noop to give the illusion of a grid
            None => Token::Noop,
        })
    }

    fn move_program_pointer(&mut self) {
        let max_y = self.grid.len() as i32;
        let max_x = self.grid[self.yptr as usize].len() as i32;

        match self.direction {
            Direction::Up    => self.yptr = decrement_wrap(self.yptr, max_y),
            Direction::Down  => self.yptr = increment_wrap(self.yptr, max_y),
            Direction::Left  => self.xptr = decrement_wrap(self.xptr, max_x),
            Direction::Right => self.xptr = increment_wrap(self.xptr, max_x),
        };
    }

    fn perform_action(&mut self, action: Token) {
        match action {
            Token::Add          => self.binary_stack_op_push(|a, b| a + b),
            Token::Subtract     => self.binary_stack_op_push(|a, b| b - a),
            Token::Multiply     => self.binary_stack_op_push(|a, b| a * b),
            Token::Divide       => {
                let a = self.stack_pop();
                let b = self.stack_pop();

                self.stack.push(
                    if a == 0 {
                        self.input_reader.read_int()
                    } else {
                        b / a
                    }
                );
            },
            Token::Modulo       => self.binary_stack_op_push(|a, b| b % a),
            Token::Not          => {
                let stack_val = self.stack_pop();
                self.stack_push(if stack_val == 0 { 1 } else { 0 });
            },
            Token::Greater      => self.binary_stack_op_push(|a, b| if b > a { 1 } else { 0 }),
            Token::Right        => self.direction = Direction::Right,
            Token::Left         => self.direction = Direction::Left,
            Token::Up           => self.direction = Direction::Up,
            Token::Down         => self.direction = Direction::Down,
            Token::Random       => self.direction = rand::random(),
            Token::HorizontalIf => {
                self.direction = if self.stack_pop() == 0 {
                    Direction::Right
                } else {
                    Direction::Left
                }
            },
            Token::VerticalIf   => {
                self.direction = if self.stack_pop() == 0 {
                    Direction::Down
                } else {
                    Direction::Up
                }
            },
            Token::StringMode   => self.string_mode = true,
            Token::Duplicate    => self.stack_push(self.stack_peek()),
            Token::Swap         => {
                let top = self.stack_pop();
                let bottom = self.stack_pop();
                self.stack_push(top);
                self.stack_push(bottom);
            },
            Token::Discard      => { self.stack_pop(); },
            Token::PrintInt     => self.last_output = format!("{} ", self.stack_pop()),
            Token::PrintChar    => self.last_output = format!("{}", i32_to_char(self.stack_pop())),
            Token::Bridge       => self.move_program_pointer(),
            Token::Get          => {
                let y = self.stack_pop();
                let x = self.stack_pop();
                self.stack_push(match self.get_token(x, y) {
                    Some(token) => token_to_char(&token) as i32,
                    None        => 0,
                });
            },
            Token::Put          => {
                let y = self.stack_pop();
                let x = self.stack_pop();
                let v = self.stack_pop();
                self.set_token(x, y, char_to_token(i32_to_char(v)));
            },
            Token::ReadInt      => {
                let int = self.input_reader.read_int();
                self.stack_push(int);
            },
            Token::ReadChar     => {
                let character = self.input_reader.read_char();
                self.stack_push(character);
            },
            Token::Quit         => self.is_running = false,
            Token::Int(value)   => self.stack.push(value as i32),
            Token::Noop         => {}, // Do nothing
            Token::Char(_)      => {}, // Do nothing
        };
    }

    fn perform_string_action(&mut self, action: Token) {
        match action {
            Token::StringMode  => self.string_mode = false,
            Token::Char(value) => self.stack_push(value as i32),
            token => self.stack_push(token_to_char(&token) as i32),
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn get_last_output(&self) -> String {
        self.last_output.clone()
    }

    pub fn xptr(&self) -> i32 {
        self.xptr
    }

    pub fn yptr(&self) -> i32 {
        self.yptr
    }

    pub fn get_stack(&self) -> &Vec<i32> {
        &self.stack
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let program_str = self.grid.iter()
            .map(|line| line.iter()
                 .map(token_to_char)
                 .collect::<String>())
            .fold(String::new(), |res, line| format!("{}\n{}", res, line));

        write!(f, "{}", program_str)
    }
}
