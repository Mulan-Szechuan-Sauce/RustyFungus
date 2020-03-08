use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::char;
use bimap::BiMap;
use std::iter::FromIterator;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

#[macro_use]
extern crate lazy_static;

fn exit_with_message(message: &str) {
    eprintln!("{}", message);
    std::process::exit(1);
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum Token {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Not,
    Greater,
    Right,
    Left,
    Up,
    Down,
    Random,
    HorizontalIf,
    VerticalIf,
    StringMode,
    Duplicate,
    Swap,
    Discard,
    PrintInt,
    PrintChar,
    Bridge,
    Get,
    Put,

    Quit,
    Int(u8),
    Noop,
    Char(char),
}

lazy_static! {
    static ref CHAR_TOKEN_MAP: bimap::hash::BiHashMap<char, Token> = BiMap::from_iter(vec![
        ('+', Token::Add),
        ('-', Token::Subtract),
        ('*', Token::Multiply),
        ('/', Token::Divide),
        ('%', Token::Modulo),
        ('!', Token::Not),
        ('`', Token::Greater),
        ('>', Token::Right),
        ('<', Token::Left),
        ('^', Token::Up),
        ('v', Token::Down),
        ('?', Token::Random),
        ('_', Token::HorizontalIf),
        ('|', Token::VerticalIf),
        ('"', Token::StringMode),
        (':', Token::Duplicate),
        ('\\',Token::Swap),
        ('$', Token::Discard),
        ('.', Token::PrintInt),
        (',', Token::PrintChar),
        ('#', Token::Bridge),
        ('g', Token::Get),
        ('p', Token::Put),
        ('@', Token::Quit),
        (' ', Token::Noop),
    ]);
}

#[derive(Copy, Clone)]
enum Direction {
    Up,
    Down,
    Right,
    Left,
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0, 4) {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            _ => Direction::Right,
        }
    }
}

struct Program {
    xptr: i32,
    yptr: i32,
    direction: Direction,
    grid: Vec<Vec<Token>>,
    stack: Vec<i32>,
    is_running: bool,
    string_mode: bool,
    _width: i32,
}

impl Program {
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
        self._width
    }
}

// TODO: Bold the current point, perhaps?

fn token_to_char(token: &Token) -> char {
    match token {
        Token::Int(value)   => (value + ('0' as u8)) as char,
        Token::Char(value)  => *value,
        value               => *CHAR_TOKEN_MAP.get_by_right(&value).unwrap(),
    }
}

fn char_to_token(character: char) -> Token {
    match character {
        '0'..='9' => Token::Int(character.to_digit(10).unwrap() as u8),
        value     => match CHAR_TOKEN_MAP.get_by_left(&value) {
            Some(c) => *c,
            None    => Token::Char(value),
        }
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

fn set_token(program: &mut Program, x: i32, y: i32, token: Token) {
    if x < 0 || y < 0  { // TODO: Be fancy and add rows/columns to the top/left
        panic!("Tried setting a negative value on the grid");
    }

    if x > program.width() {
        program._width = x;
    }

    let x = x as usize;
    let y = y as usize;

    // Expand the height to allow inserting at coordinate y
    for _ in program.grid.len()..=y {
        program.grid.push(vec![]);
    }

    // Expand row with no-ops to make room for the new token
    for _ in program.grid[y].len()..=x {
        program.grid[y].push(Token::Noop);
    }

    program.grid[y][x] = token;
}

fn get_token(program: &Program, x: i32, y: i32) -> Option<Token> {
    if x < 0 || y < 0 || x >= program.width() || y >= program.height() {
        return None;
    }

    let x = x as usize;
    let y = y as usize;

    Some(match program.grid.get(y) {
        Some(row) => match row.get(x) {
            Some(token) => token.clone(),
            None => Token::Noop,
        },
        // Default to Noop to give the illusion of a grid
        None => Token::Noop,
    })
}

fn lines_to_token_matrix(lines: std::str::Lines) -> Vec<Vec<Token>> {
    lines.map(|line| {
        line.chars().map(|c| char_to_token(c)).collect()
    }).collect()
}

fn load_program(filename: String) -> Result<Program, io::Error> {
    let contents = fs::read_to_string(filename)?;

    let parsed_contents = lines_to_token_matrix(contents.lines());
    let max_width = parsed_contents.iter()
        .map(|line| line.len())
        .max()
        .unwrap_or(0) as i32;

    return Ok(Program {
        xptr: 0,
        yptr: 0,
        direction: Direction::Right,
        grid: parsed_contents,
        stack: vec![],
        is_running: true,
        string_mode: false,
        _width: max_width,
    });
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

fn move_program_pointer(program: &mut Program) {
    let max_y = program.grid.len() as i32;
    let max_x = program.grid[program.yptr as usize].len() as i32;

    match program.direction {
        Direction::Up    => program.yptr = decrement_wrap(program.yptr, max_y),
        Direction::Down  => program.yptr = increment_wrap(program.yptr, max_y),
        Direction::Left  => program.xptr = decrement_wrap(program.xptr, max_x),
        Direction::Right => program.xptr = increment_wrap(program.xptr, max_x),
    };
}

fn i32_to_char(value: i32) -> char {
    char::from_u32(value as u32).unwrap()
}


fn perform_action(program: &mut Program, action: Token) {
    match action {
        Token::Add          => program.binary_stack_op_push(|a, b| a + b),
        Token::Subtract     => program.binary_stack_op_push(|a, b| b - a),
        Token::Multiply     => program.binary_stack_op_push(|a, b| a * b),
        Token::Divide       => program.binary_stack_op_push(|a, b| b / a),
        Token::Modulo       => program.binary_stack_op_push(|a, b| b % a),
        Token::Not          => {
            let stack_val = program.stack_pop();
            program.stack_push(if stack_val == 0 { 1 } else { 0 });
        },
        Token::Greater      => program.binary_stack_op_push(|a, b| if b > a { 1 } else { 0 }),
        Token::Right        => program.direction = Direction::Right,
        Token::Left         => program.direction = Direction::Left,
        Token::Up           => program.direction = Direction::Up,
        Token::Down         => program.direction = Direction::Down,
        Token::Random       => program.direction = rand::random(),
        Token::HorizontalIf => {
            program.direction = if program.stack_pop() == 0 {
                Direction::Right
            } else {
                Direction::Left
            }
        },
        Token::VerticalIf   => {
            program.direction = if program.stack_pop() == 0 {
                Direction::Down
            } else {
                Direction::Up
            }
        },
        Token::StringMode   => program.string_mode = true,
        Token::Duplicate    => program.stack_push(program.stack_peek()),
        Token::Swap         => {
            let top = program.stack_pop();
            let bottom = program.stack_pop();
            program.stack_push(top);
            program.stack_push(bottom);
        },
        Token::Discard      => { program.stack_pop(); },
        Token::PrintInt     => print!("{} ", program.stack_pop()),
        Token::PrintChar    => print!("{}", i32_to_char(program.stack_pop())),
        Token::Bridge       => move_program_pointer(program),
        Token::Get          => {
            let y = program.stack_pop();
            let x = program.stack_pop();
            program.stack_push(match get_token(program, x, y) {
                Some(token) => token_to_char(&token) as i32,
                None        => 0,
            });
        },
        Token::Put          => {
            let y = program.stack_pop();
            let x = program.stack_pop();
            let v = program.stack_pop();
            set_token(program, x, y, char_to_token(i32_to_char(v)));
        },
        Token::Quit         => program.is_running = false,
        Token::Int(value)   => program.stack.push(value as i32),
        Token::Noop         => {}, // Do nothing
        Token::Char(_)      => {}, // Do nothing
    };
}

fn perform_string_action(program: &mut Program, action: Token) {
    match action {
        Token::StringMode  => program.string_mode = false,
        Token::Char(value) => program.stack_push(value as i32),
        token => program.stack_push(*CHAR_TOKEN_MAP.get_by_right(&token).unwrap() as i32),
    }
}

fn step_program(mut program: &mut Program) {
    let current_token = get_token(program, program.xptr, program.yptr).unwrap();
    if program.string_mode {
        perform_string_action(program, current_token);
    } else {
        perform_action(program, current_token);
    }
    move_program_pointer(&mut program);
}

fn run_program(mut program: &mut Program) {
    while program.is_running {
        step_program(&mut program);
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
