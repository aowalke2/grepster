use std::{char, collections::HashMap};

#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub matched_text: String,
    pub sub_matches: HashMap<usize, PatternMatch>,
}

impl PatternMatch {
    fn new(matched_text: &str) -> Self {
        Self {
            matched_text: matched_text.to_string(),
            sub_matches: HashMap::new(),
        }
    }

    fn accumulate(&mut self, other: &PatternMatch) {
        self.matched_text.push_str(&other.matched_text);
        for (group_idx, sub_match) in &other.sub_matches {
            self.sub_matches.insert(*group_idx, sub_match.clone());
        }
    }
}

#[derive(Clone, Debug)]
pub enum Pattern {
    Literal(char),
    Digit,
    Alphanumeric,
    PositiveCharacterGroup(Vec<char>),
    NegativeCharacterGroup(Vec<char>),
    OneOrMore {
        pattern: Box<Pattern>,
        next_pattern: Option<Box<Pattern>>,
    },
    ZeroOrOne(Box<Pattern>),
    Wildcard,
    Start,
    End,
    Sequence(Vec<Pattern>),
    Group(Vec<Pattern>, usize),
    GroupReference(usize),
}

impl Pattern {
    pub fn match_pattern(&self, input: &str) -> bool {
        for index in 0..input.chars().count() {
            match self.match_here(input, index, &HashMap::new()) {
                Some(_) => return true,
                None => continue,
            }
        }
        false
    }

    fn match_here(
        &self,
        input: &str,
        index: usize,
        matches: &HashMap<usize, String>,
    ) -> Option<PatternMatch> {
        use Pattern::*;
        match self {
            Literal(literal) => match input.chars().nth(index) {
                Some(c) => match c == *literal {
                    true => Some(PatternMatch::new(&literal.to_string())),
                    false => None,
                },
                None => None,
            },
            Digit => match input.chars().nth(index) {
                Some(c) => match c.is_numeric() {
                    true => Some(PatternMatch::new(&c.to_string())),
                    false => return None,
                },
                None => None,
            },
            Alphanumeric => match input.chars().nth(index) {
                Some(c) => match c.is_alphanumeric() {
                    true => Some(PatternMatch::new(&c.to_string())),
                    false => return None,
                },
                None => None,
            },
            PositiveCharacterGroup(chars) => match input.chars().nth(index) {
                Some(ch) => match chars.contains(&ch) {
                    true => Some(PatternMatch::new(&ch.to_string())),
                    false => None,
                },
                None => None,
            },
            NegativeCharacterGroup(chars) => match input.chars().nth(index) {
                Some(ch) => match !chars.contains(&ch) {
                    true => Some(PatternMatch::new(&ch.to_string())),
                    false => None,
                },
                None => None,
            },
            OneOrMore {
                pattern,
                next_pattern,
            } => {
                let mut i = index;
                let mut curr_pattern_match = PatternMatch::new("");
                let mut curr_matches = matches.clone();

                loop {
                    match pattern.match_here(input, i, &curr_matches) {
                        Some(pattern_match) => {
                            if !curr_pattern_match.matched_text.is_empty()
                                && next_pattern.as_ref().is_some_and(|next_pattern| {
                                    next_pattern.match_pattern(&pattern_match.matched_text)
                                })
                            {
                                return Some(curr_pattern_match);
                            }

                            curr_pattern_match.accumulate(&pattern_match);
                            i += pattern_match.matched_text.chars().count();
                            Self::update_matches(&mut curr_matches, &pattern_match);
                        }
                        None => match curr_pattern_match.matched_text.is_empty() {
                            true => return None,
                            false => break,
                        },
                    }
                }

                Some(curr_pattern_match)
            }
            ZeroOrOne(pattern) => {
                let mut m = PatternMatch::new("");
                match pattern.match_here(input, index, matches) {
                    Some(other) => m.accumulate(&other),
                    None => {}
                }
                Some(m)
            }
            Wildcard => input
                .chars()
                .nth(index)
                .map(|c| PatternMatch::new(&c.to_string())),

            Start => match index == 0 {
                true => Some(PatternMatch::new("")),
                false => None,
            },
            End => match index == input.len() {
                true => Some(PatternMatch::new("")),
                false => None,
            },
            Sequence(patterns) => {
                let mut curr_offset = index;
                let mut curr_groups = matches.clone();
                let mut pattern_match = PatternMatch::new("");

                for pattern in patterns {
                    match pattern.match_here(input, curr_offset, &curr_groups) {
                        Some(other) => {
                            pattern_match.accumulate(&other);
                            curr_offset += other.matched_text.chars().count();
                            Self::update_matches(&mut curr_groups, &other);
                        }
                        None => return None,
                    }
                }
                Some(pattern_match)
            }
            Group(patterns, pattern_index) => {
                let mut pattern_match = PatternMatch::new("");
                for pattern in patterns {
                    if let Some(other) = pattern.match_here(input, index, matches) {
                        pattern_match.accumulate(&other);
                        pattern_match
                            .sub_matches
                            .insert(*pattern_index, pattern_match.clone());
                        return Some(pattern_match);
                    }
                }
                None
            }
            GroupReference(pattern_index) => match matches.get(&pattern_index) {
                Some(matched_text) => {
                    let input = input.chars().skip(index).collect::<String>();
                    if input.starts_with(matched_text) {
                        Some(PatternMatch::new(&matched_text))
                    } else {
                        None
                    }
                }
                None => None,
            },
        }
    }

    fn update_matches(matches: &mut HashMap<usize, String>, m: &PatternMatch) {
        for (i, sub_match) in &m.sub_matches {
            matches.insert(*i, sub_match.matched_text.clone());
        }
    }
}
