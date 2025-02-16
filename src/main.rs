use std::env;
use std::io;
use std::process;
use std::str::Chars;

pub enum Pattern {
    Literal(char),
    Digit,
    Alphanumeric,
    CharacterGroup(bool, String),
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let patterns = parse_patterns(pattern);
    let input_line = input_line.trim_matches('\n');
    'outer: for i in 0..input_line.len() {
        let chars = &mut input_line[i..].chars();
        for pattern in patterns.iter() {
            match pattern {
                Pattern::Literal(literal) => {
                    if !match_literal(chars, *literal) {
                        continue 'outer;
                    }
                }
                Pattern::Digit => {
                    if !match_digit(chars) {
                        continue 'outer;
                    }
                }
                Pattern::Alphanumeric => {
                    if !match_alphanumeric(chars) {
                        continue 'outer;
                    }
                }
                Pattern::CharacterGroup(is_positive, group) => {
                    if match_group(chars, group) != *is_positive {
                        continue 'outer;
                    }
                }
            }
        }
        return true;
    }
    return false;
}

fn match_literal(chars: &mut Chars, literal: char) -> bool {
    chars.next().is_some_and(|c| c == literal)
}

fn match_digit(chars: &mut Chars) -> bool {
    chars.next().is_some_and(|c| c.is_numeric())
}

fn match_alphanumeric(chars: &mut Chars) -> bool {
    chars.next().is_some_and(|c| c.is_alphanumeric())
}

fn match_group(chars: &mut Chars, group: &str) -> bool {
    chars.next().is_some_and(|c| group.contains(c))
}

fn parse_patterns(pattern: &str) -> Vec<Pattern> {
    let mut chars = pattern.chars();
    let mut patterns = Vec::new();

    while let Some(c) = chars.next() {
        let pattern = match c {
            '\\' => {
                let special = chars.next();
                if special.is_none() {
                    panic!("Incomplete special character")
                }
                match special.unwrap() {
                    'd' => Pattern::Digit,
                    'w' => Pattern::Alphanumeric,
                    '\\' => Pattern::Literal('\\'),
                    _ => panic!("Invalid special character"),
                }
            }
            '[' => {
                let (is_positive, group) = parse_character_group(&mut chars);
                Pattern::CharacterGroup(is_positive, group)
            }
            c => Pattern::Literal(c),
        };
        patterns.push(pattern);
    }

    patterns
}

fn parse_character_group(chars: &mut Chars) -> (bool, String) {
    let mut group = String::new();
    let mut is_positvie = true;

    if chars.next().is_some_and(|c| c == '^') {
        is_positvie = false;
        chars.next();
    }

    while let Some(c) = chars.next() {
        if c != ']' {
            group.push(c);
        } else {
            break;
        }
    }
    (is_positvie, group)
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
