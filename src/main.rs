use std::env;
use std::fmt;
use std::fs;
use std::io;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

fn exit_with_message(message: &str) {
    eprintln!("{}", message);
    std::process::exit(1);
}

#[derive(Copy, Clone)]
enum Token {
    Noop,
    Up,
    Down,
    Right,
    Left,
    Random,
    PrintInt,
    Int(u8),
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
    xptr: usize,
    yptr: usize,
    direction: Direction,
    grid: Vec<Vec<Token>>,
    stack: Vec<i32>,
    is_running: bool,
}

// TODO: Bold the current point, perhaps?

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = self.grid.iter()
            .map(|line| line.iter()
                 .map(|token| match token {
                     Token::Noop       => ' ',
                     Token::Up         => '^',
                     Token::Down       => 'v',
                     Token::Right      => '>',
                     Token::Left       => '<',
                     Token::Random     => '?',
                     Token::PrintInt   => '.',
                     Token::Int(value) => (value + ('0' as u8) - 1) as char,
                 })
                 .collect::<String>())
            .fold(String::new(), |res, line| format!("{}\n{}", res, line));

        write!(f, "{}", result)
    }
}

fn set_token(program: &mut Program, x: usize, y: usize, token: Token) {
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

fn get_token(program: &Program, x: usize, y: usize) -> Token {
    match program.grid.get(y) {
        Some(row) => match row.get(x) {
            Some(token) => token.clone(),
            None => Token::Noop,
        },
        // Default to Noop to give the illusion of an infinite grid
        None => Token::Noop,
    }
}

fn lines_to_token_matrix(lines: std::str::Lines) -> Vec<Vec<Token>> {
    lines.map(|line| {
        line.chars().map(|c| match c {
            ' ' => Token::Noop,
            '^' => Token::Up,
            'v' => Token::Down,
            '>' => Token::Right,
            '<' => Token::Left,
            '?' => Token::Random,
            '.' => Token::PrintInt,
            '0'..='9' => Token::Int(c.to_digit(10).unwrap() as u8),
            _ => panic!("Invalid character found"),
        }).collect()
    }).collect()
}

fn load_program(filename: String) -> Result<Program, io::Error> {
    let contents = fs::read_to_string(filename)?;

    return Ok(Program {
        xptr: 0,
        yptr: 0,
        direction: Direction::Right,
        grid: lines_to_token_matrix(contents.lines()),
        stack: vec![],
        is_running: true,
    });
}

fn increment_wrap(value: usize, max_value: usize) -> usize {
    if value == max_value - 1 {
        0
    } else {
        value + 1
    }
}

fn decrement_wrap(value: usize, max_value: usize) -> usize {
    if value == 0 {
        max_value - 1
    } else {
        value - 1
    }
}

fn move_program_pointer(program: &mut Program) {
    let max_y = program.grid.len();
    let max_x = program.grid[program.yptr].len();

    match program.direction {
        Direction::Up    => program.yptr = decrement_wrap(program.yptr, max_y),
        Direction::Down  => program.yptr = increment_wrap(program.yptr, max_y),
        Direction::Left  => program.xptr = decrement_wrap(program.xptr, max_x),
        Direction::Right => program.xptr = increment_wrap(program.xptr, max_x),
    };
}

fn perform_action(program: &mut Program, action: Token) {
    match action {
        Token::Noop       => {}, // Do nothing
        Token::Up         => program.direction = Direction::Up,
        Token::Down       => program.direction = Direction::Down,
        Token::Right      => program.direction = Direction::Right,
        Token::Left       => program.direction = Direction::Left,
        Token::Random     => program.direction = rand::random(),
        Token::Int(value) => program.stack.push(value as i32),
        Token::PrintInt   => match program.stack.pop() {
            Some(value) => print!("{} ", value),
            None => {
                println!("Hit bottom of stack");
                program.is_running = false;
            },
        },
    };
}

fn step_program(mut program: &mut Program) {
    let current_token = get_token(program, program.xptr, program.yptr);
    perform_action(program, current_token);
    move_program_pointer(&mut program);
}

fn run_program(mut program: Program) {
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
                Ok(program) => run_program(program),
                Err(e) => exit_with_message(&e.to_string()),
            }
        },
        _ => exit_with_message("Usage: rustyfunges <filename>"),
    }
}
