use std::env;
use std::io;
use std::process;
use std::str::Chars;

#[derive(Debug)]
pub enum Pattern {
    Literal(char),
    Digit,
    Alphanumeric,
    CharacterGroup(bool, String),
    Start(String),
    End(String),
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let patterns = parse_patterns(pattern);
    let input_line = input_line.trim_matches('\n');
    'outer: for i in 0..input_line.len() {
        let chars = &mut input_line[i..].chars();
        for pattern in patterns.iter() {
            match pattern {
                Pattern::Literal(literal) => {
                    if !chars.next().is_some_and(|c| c == *literal) {
                        continue 'outer;
                    }
                }
                Pattern::Digit => {
                    if !chars.next().is_some_and(|c| c.is_numeric()) {
                        continue 'outer;
                    }
                }
                Pattern::Alphanumeric => {
                    if !chars.next().is_some_and(|c| c.is_alphanumeric()) {
                        continue 'outer;
                    }
                }
                Pattern::CharacterGroup(is_positive, group) => {
                    if chars.next().is_some_and(|c| group.contains(c)) != *is_positive {
                        continue 'outer;
                    }
                }
                Pattern::Start(string) => return chars.collect::<String>().as_str() == string,
                Pattern::End(string) => return chars.collect::<String>().as_str() == string,
            }
        }
        return true;
    }
    return false;
}

fn parse_patterns(pattern: &str) -> Vec<Pattern> {
    let mut patterns = Vec::new();
    if pattern.starts_with('^') {
        patterns.push(Pattern::Start(pattern.chars().skip(1).collect()));
        return patterns;
    }

    if pattern.ends_with('$') {
        patterns.push(Pattern::End(
            pattern.chars().take(pattern.chars().count() - 1).collect(),
        ));
        return patterns;
    }

    let mut chars = pattern.chars();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                let special = chars.next();
                if special.is_none() {
                    panic!("Incomplete special character")
                }
                match special.unwrap() {
                    'd' => patterns.push(Pattern::Digit),
                    'w' => patterns.push(Pattern::Alphanumeric),
                    '\\' => patterns.push(Pattern::Literal('\\')),
                    _ => panic!("Invalid special character"),
                }
            }
            '[' => {
                let (is_positive, group) = parse_character_group(&mut chars);
                patterns.push(Pattern::CharacterGroup(is_positive, group))
            }
            c => patterns.push(Pattern::Literal(c)),
        };
    }

    patterns
}

fn parse_character_group(chars: &mut Chars) -> (bool, String) {
    let mut group = String::new();
    let mut is_positive = true;

    if chars.clone().next().is_some_and(|c| c == '^') {
        is_positive = false;
        chars.next();
    }

    while let Some(c) = chars.next() {
        if c != ']' {
            group.push(c);
        } else {
            break;
        }
    }
    (is_positive, group)
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
        println!("0");
        process::exit(0)
    } else {
        println!("1");
        process::exit(1)
    }
}
