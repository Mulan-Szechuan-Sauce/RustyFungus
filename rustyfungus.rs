use std::env;
use std::fs;
use std::fmt;

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

struct Program {
    xptr: usize,
    yptr: usize,
    grid: Vec<Vec<Token>>,
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

fn load_program(filename: String) -> Program {
    let file_contents = fs::read_to_string(filename)
        .expect("Couldn't read file");

    let mut program = Program {
        xptr: 0,
        yptr: 0,
        grid: vec![],
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
fn step_program(program: & Program) -> bool {
    // TODO:
    return false;
}

fn run_program(program: Program) {
    println!("Program: \n{}", program);

    while step_program(&program) {
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() == 2 {
        let filename = args.get(1).unwrap().to_string();
        let program = load_program(filename);
        run_program(program);
    } else {
        panic!("Usage: rustyfunges <filename>")
    }
}
