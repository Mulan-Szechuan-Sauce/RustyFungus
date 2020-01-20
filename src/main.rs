use std::env;
use std::fs;
use std::fmt;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

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
    // FIXME: Make the width dynamic too!
    let width = 128;

    // Expand the height to allow inserting at coordinate y
    for _ in (program.grid.len())..=y {
        let new_row = vec![Token::Noop; width];
        program.grid.push(new_row);
    }

    program.grid[y][x] = token;
}

fn get_token(program: &Program, x: usize, y: usize) -> Token {
    match program.grid.get(y) {
        Some(row) => match row.get(x) {
            Some(token) => token.clone(),
            None => Token::Noop,
        },
        None => Token::Noop,
    }
}

fn load_program(filename: String) -> Program {
    let file_contents = fs::read_to_string(filename)
        .expect("Couldn't read file");

    let mut program = Program {
        xptr: 0,
        yptr: 0,
        direction: Direction::Right,
        grid: vec![],
        stack: vec![],
    };

    let mut y = 0;

    for line in file_contents.lines() {
        let mut x = 0;

        for c in line.chars() {
            let token = match c {
                ' ' => Token::Noop,
                '^' => Token::Up,
                'v' => Token::Down,
                '>' => Token::Right,
                '<' => Token::Left,
                '?' => Token::Random,
                '.' => Token::PrintInt,
                '0'..='9' => Token::Int(c.to_digit(10).unwrap() as u8),
                _ => panic!("Invalid character found"),
            };

            set_token(&mut program, x, y, token);
            x += 1;
        }

        y += 1;
    }

    return program;
}

/**
 * Returns false when the program should end
 */
fn step_program(program: &mut Program) -> bool {
    match get_token(program, program.xptr, program.yptr) {
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
                return false;
            },
        },
    }

    match program.direction {
        Direction::Up    => program.yptr -= 1,
        Direction::Down  => program.yptr += 1,
        Direction::Left  => program.xptr -= 1,
        Direction::Right => program.xptr += 1,
    }

    return true;
}

fn run_program(mut program: &mut Program) {
    println!("Program: \n{}", &program);

    while step_program(&mut program) {
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() == 2 {
        let filename = args.get(1).unwrap().to_string();
        let mut program = load_program(filename);
        run_program(&mut program);
    } else {
        panic!("Usage: rustyfunges <filename>")
    }
}
