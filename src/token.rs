use bimap::BiMap;
use std::iter::FromIterator;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Token {
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

pub fn token_to_char(token: &Token) -> char {
    match token {
        Token::Int(value)   => (value + ('0' as u8)) as char,
        Token::Char(value)  => *value,
        value               => *CHAR_TOKEN_MAP.get_by_right(&value).unwrap(),
    }
}

pub fn char_to_token(character: char) -> Token {
    match character {
        '0'..='9' => Token::Int(character.to_digit(10).unwrap() as u8),
        value     => match CHAR_TOKEN_MAP.get_by_left(&value) {
            Some(c) => *c,
            None    => Token::Char(value),
        }
    }
}
