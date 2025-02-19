use anyhow::*;

use crate::pattern::Pattern;

pub struct Parser {
    pattern: Vec<char>,
    current_index: usize,
    next_index: usize,
}

impl Parser {
    pub fn new(pattern: &str, next_index: usize) -> Parser {
        Parser {
            pattern: pattern.chars().collect(),
            current_index: 0,
            next_index,
        }
    }

    pub fn parse(&mut self) -> Result<Pattern> {
        let mut patterns = vec![];

        while let Some(&c) = self.pattern.get(self.current_index) {
            let pattern = match c {
                '\\' => {
                    self.current_index += 1;
                    let next = self
                        .pattern
                        .get(self.current_index)
                        .ok_or(anyhow!("Expected escaped char"))?;
                    let pattern = match next {
                        'd' => Pattern::Digit,
                        'w' => Pattern::Alphanumeric,
                        '\\' | '+' | '?' | '.' | '[' | '(' => Pattern::Literal(*next),
                        _ => match next.is_numeric() {
                            true => {
                                let pattern_index = next.to_digit(10).unwrap() as usize;
                                if pattern_index >= self.next_index {
                                    return Err(anyhow!("Index out of range"));
                                }
                                Pattern::GroupReference(pattern_index)
                            }
                            false => return Err(anyhow!("Invalid character '{}'", next)),
                        },
                    };
                    self.current_index += 1;
                    pattern
                }
                '[' => self.parse_character_group()?,
                '(' => self.parse_group()?,
                '^' => {
                    self.current_index += 1;
                    Pattern::Start
                }
                '$' => {
                    self.current_index += 1;
                    Pattern::End
                }
                '+' => {
                    self.current_index += 1;
                    let last_pattern = patterns
                        .iter()
                        .last()
                        .cloned()
                        .ok_or(anyhow!("One or more expects previous char"))?;
                    let one_or_more = Pattern::OneOrMore {
                        pattern: Box::new(last_pattern),
                        next_pattern: None,
                    };
                    patterns.pop();
                    one_or_more
                }
                '?' => {
                    self.current_index += 1;
                    let last_pattern = patterns
                        .iter()
                        .last()
                        .ok_or(anyhow!("+ expects previous char"))?;
                    let zero_or_one = Pattern::ZeroOrOne(Box::new(last_pattern.clone()));
                    patterns.pop();
                    zero_or_one
                }
                '.' => {
                    self.current_index += 1;
                    Pattern::Wildcard
                }
                _ => {
                    self.current_index += 1;
                    Pattern::Literal(c)
                }
            };

            let prev_pattern = patterns.iter().last();
            let next_prev_pattern = match prev_pattern {
                Some(prev_pattern) => match prev_pattern {
                    Pattern::OneOrMore {
                        pattern: p,
                        next_pattern: _,
                    } => Some(Pattern::OneOrMore {
                        pattern: p.clone(),
                        next_pattern: Some(Box::new(pattern.clone())),
                    }),
                    Pattern::Group(patterns, pattern_index) => {
                        self.parse_group_with_one_or_more(patterns, *pattern_index, &pattern)
                    }
                    _ => None,
                },
                None => None,
            };
            if next_prev_pattern.is_some() {
                patterns.pop();
                patterns.push(next_prev_pattern.unwrap());
            }
            patterns.push(pattern);
        }

        match patterns.len() {
            0 => Err(anyhow!("No pattern found")),
            1 => Ok(patterns[0].clone()),
            _ => Ok(Pattern::Sequence(patterns)),
        }
    }

    fn parse_character_group(&mut self) -> Result<Pattern> {
        let mut chars = vec![];
        let mut is_positive = true;

        self.current_index += 1;
        let c = self
            .pattern
            .get(self.current_index)
            .ok_or(anyhow!("Expected character"))?;

        match *c == '^' {
            true => is_positive = false,
            false => chars.push(*c),
        }
        self.current_index += 1;

        while let Some(&c) = self.pattern.get(self.current_index) {
            self.current_index += 1;
            if c == ']' {
                break;
            }
            chars.push(c);
        }

        match is_positive {
            true => Ok(Pattern::PositiveCharacterGroup(chars)),
            false => Ok(Pattern::NegativeCharacterGroup(chars)),
        }
    }

    fn parse_group_with_one_or_more(
        &self,
        patterns: &Vec<Pattern>,
        pattern_index: usize,
        next_pattern: &Pattern,
    ) -> Option<Pattern> {
        let mut new_patterns = vec![];
        let mut updated = false;

        for pattern in patterns {
            let new_pattern = match pattern {
                Pattern::OneOrMore {
                    pattern: p,
                    next_pattern: _,
                } => {
                    updated = true;
                    Pattern::OneOrMore {
                        pattern: p.clone(),
                        next_pattern: Some(Box::new(next_pattern.clone())),
                    }
                }
                Pattern::Group(patterns, patten_index) => {
                    match self.parse_group_with_one_or_more(patterns, *patten_index, &pattern) {
                        Some(new_group) => {
                            updated = true;
                            new_group
                        }
                        None => pattern.clone(),
                    }
                }
                _ => pattern.clone(),
            };
            new_patterns.push(new_pattern);
        }

        match updated {
            true => Some(Pattern::Group(new_patterns, pattern_index)),
            false => None,
        }
    }

    fn parse_group(&mut self) -> Result<Pattern> {
        let (chunks, consumed) = self.split_alternation()?;
        let mut patterns = vec![];
        let next_index = self.next_index;
        self.next_index += 1;

        for chunk in &chunks {
            let mut parser = Parser::new(chunk, self.next_index);
            let pattern = parser.parse()?;
            patterns.push(pattern);
            self.next_index = parser.next_index;
        }
        self.current_index += consumed;

        Ok(Pattern::Group(patterns, next_index))
    }

    fn split_alternation(&self) -> Result<(Vec<String>, usize)> {
        let mut chunks = vec![];
        let mut chunk = String::new();
        let mut level = 0;
        let mut consumed = 0;

        for (i, c) in self.pattern[self.current_index..].iter().enumerate() {
            match *c {
                '(' => {
                    level += 1;
                    if level == 1 {
                        continue;
                    }
                }
                ')' => {
                    level -= 1;
                    if level == 0 {
                        if chunk.is_empty() {
                            return Err(anyhow!("Empty alternation"));
                        }
                        chunks.push(chunk.clone());
                        chunk.clear();
                        consumed = i + 1;
                        break;
                    }
                }
                '|' => {
                    if level == 1 {
                        if chunk.is_empty() {
                            return Err(anyhow!("Empty alternation"));
                        }
                        chunks.push(chunk.clone());
                        chunk.clear();
                        continue;
                    }
                }
                _ => {}
            }
            chunk.push(*c);
        }

        if !chunk.is_empty() || level != 0 {
            return Err(anyhow!("Invalid alternation pattern"));
        }

        Ok((chunks, consumed))
    }
}
