use std::env;
use std::io;
use std::iter::Peekable;
use std::process;
use std::slice;
use std::str::Chars;

#[derive(Debug, PartialEq, Eq)]
pub enum Pattern {
    Literal(char),
    OneOrMore(char),
    Digit,
    Alphanumeric,
    PositiveCharacterGroup(String),
    NegativeCharacterGroup(String),
    Start,
    End,
}

#[derive(Debug)]
pub struct Grep {
    input: String,
    patterns: Vec<Pattern>,
}

impl Grep {
    pub fn new(input_line: &str, pattern: &str) -> Grep {
        let input = input_line.to_string();
        let patterns = parse_patterns(pattern);
        Grep { input, patterns }
    }

    pub fn match_pattern(&self) -> bool {
        if self.patterns[0] == Pattern::Start {
            return match_here(
                &mut self.input.chars().peekable(),
                &mut self.patterns[1..].iter(),
            );
        }

        for i in 0..self.input.len() {
            if match_here(
                &mut self.input[i..].chars().peekable(),
                &mut self.patterns.iter(),
            ) {
                return true;
            }
        }

        false
    }
}

fn match_here(input: &mut Peekable<Chars>, patterns: &mut slice::Iter<Pattern>) -> bool {
    match patterns.next() {
        Some(Pattern::Literal(literal)) => {
            input.next_if_eq(literal).is_some() && match_here(input, patterns)
        }
        Some(Pattern::Digit) => {
            input.next_if(|&c| c.is_numeric()).is_some() && match_here(input, patterns)
        }
        Some(Pattern::Alphanumeric) => {
            input.next_if(|&c| c.is_alphanumeric()).is_some() && match_here(input, patterns)
        }
        Some(Pattern::PositiveCharacterGroup(group)) => {
            input.next_if(|&c| group.contains(c)).is_some() && match_here(input, patterns)
        }
        Some(Pattern::NegativeCharacterGroup(group)) => {
            input.next_if(|&c| !group.contains(c)).is_some() && match_here(input, patterns)
        }
        Some(Pattern::Start) => false,
        Some(Pattern::End) => input.peek().map_or(true, |&c| c == '\n'),
        Some(Pattern::OneOrMore(repeated)) => {
            if input.next_if(|&c| c == *repeated).is_some() {
                while input.next_if(|&c| c == *repeated).is_some() {}
                match_here(input, patterns)
            } else {
                false
            }
        }
        None => true,
    }
}

fn parse_patterns(pattern: &str) -> Vec<Pattern> {
    let mut pattern_chars = pattern.chars().peekable();
    let mut patterns = Vec::new();

    while let Some(&c) = pattern_chars.peek() {
        match c {
            '\\' => {
                pattern_chars.next();
                if let Some(special) = pattern_chars.next() {
                    match special {
                        'd' => patterns.push(Pattern::Digit),
                        'w' => patterns.push(Pattern::Alphanumeric),
                        '\\' => patterns.push(Pattern::Literal('\\')),
                        _ => panic!("Invalid special character"),
                    }
                } else {
                    panic!("Invalid special character");
                }
            }
            '^' => {
                if !patterns.is_empty() {
                    panic!("Start anchor should be first");
                }
                pattern_chars.next();
                patterns.push(Pattern::Start);
            }
            '$' => {
                pattern_chars.next();
                if let Some(_) = pattern_chars.peek() {
                    panic!("End anchor should be last");
                }
                patterns.push(Pattern::End);
            }
            '[' => {
                pattern_chars.next();
                let mut group = String::new();
                let mut is_positive = true;

                if pattern_chars.peek() == Some(&'^') {
                    pattern_chars.next();
                    is_positive = false;
                }

                while let Some(&c) = pattern_chars.peek() {
                    match c {
                        ']' => {
                            pattern_chars.next();
                            break;
                        }
                        _ => {
                            group.push(c);
                            pattern_chars.next();
                        }
                    }
                }
                match is_positive {
                    true => patterns.push(Pattern::PositiveCharacterGroup(group)),
                    false => patterns.push(Pattern::NegativeCharacterGroup(group)),
                }
            }
            _ => {
                pattern_chars.next();
                if pattern_chars.next_if(|&c| c == '+').is_some() {
                    patterns.push(Pattern::OneOrMore(c));
                } else {
                    patterns.push(Pattern::Literal(c));
                }
            }
        };
    }

    patterns
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

    let grep = Grep::new(&input_line, &pattern);
    if grep.match_pattern() {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
