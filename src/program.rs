use crate::token::{Token, token_to_char, char_to_token};
use crate::direction::Direction;

use std::fmt;
use std::char;

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
}

impl Program {
    pub fn new(parsed_contents: Vec<Vec<Token>>) -> Program {
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

    fn height(&self) -> i32 {
        self.grid.len() as i32
    }

    fn width(&self) -> i32 {
        self.width
    }

    pub fn step(&mut self) {
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
            Token::Divide       => self.binary_stack_op_push(|a, b| b / a),
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
            Token::PrintInt     => print!("{} ", self.stack_pop()),
            Token::PrintChar    => print!("{}", i32_to_char(self.stack_pop())),
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
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = self.grid.iter()
            .map(|line| line.iter()
                 .map(token_to_char)
                 .collect::<String>())
            .fold(String::new(), |res, line| format!("{}\n{}", res, line));

        write!(f, "{}", result)
    }
}
