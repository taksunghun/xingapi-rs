// SPDX-License-Identifier: MPL-2.0

use std::str::CharIndices;

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Position {
    line: usize,
    column: usize,
}

impl Position {
    pub fn line(&self) -> usize {
        self.line
    }
    pub fn column(&self) -> usize {
        self.column
    }
}

pub trait Read<'a> {
    fn get_symbol(&mut self) -> Option<&'a str>;
    fn next_symbol(&mut self) -> Option<&'a str>;
    fn position(&self) -> Position;
}

pub struct StrRead<'a> {
    string: &'a str,
    iter: CharIndices<'a>,
    latest_offset: usize,
}

impl<'a> StrRead<'a> {
    pub fn new(string: &'a str) -> Self {
        Self { string, iter: string.char_indices(), latest_offset: 0 }
    }

    fn skip_until_not_whitespace(&mut self) -> Option<()> {
        while let Some((_, ch)) = self.iter.clone().next() {
            match ch {
                ' ' | '\t' | '\r' | '\n' => {
                    self.iter.next().unwrap();
                }
                _ => {
                    return Some(());
                }
            }
        }

        None
    }

    fn skip_until_not_comment(&mut self) -> Option<()> {
        while let Some((_, ch)) = self.iter.next() {
            if ch == '*' {
                if let Some((_, ch)) = self.iter.clone().next() {
                    if ch == '/' {
                        self.iter.next().unwrap();
                        return Some(());
                    }
                }
            }
        }

        None
    }
}

impl<'a> Read<'a> for StrRead<'a> {
    fn get_symbol(&mut self) -> Option<&'a str> {
        let iter = self.iter.clone();
        let symbol = self.next_symbol();
        self.iter = iter;

        symbol
    }

    fn next_symbol(&mut self) -> Option<&'a str> {
        self.skip_until_not_whitespace()?;

        self.latest_offset = self.iter.clone().next()?.0;
        let mut start_idx = self.latest_offset;
        let mut end_idx = self.string.len();

        while let Some((idx, ch)) = self.iter.clone().next() {
            match ch {
                '\r' | '\n' => {
                    if idx == start_idx {
                        self.skip_until_not_whitespace()?;
                        self.latest_offset = self.iter.clone().next()?.0;
                        start_idx = self.latest_offset;
                    } else {
                        end_idx = idx;
                        break;
                    }
                }
                '/' => {
                    if let Some((_, ch2)) = self.iter.clone().skip(1).next() {
                        if ch2 == '*' {
                            if idx == start_idx {
                                self.iter.next().unwrap();
                                self.iter.next().unwrap();
                                self.skip_until_not_comment()?;

                                self.latest_offset = self.iter.clone().next()?.0;
                                start_idx = self.latest_offset;
                                continue;
                            } else {
                                end_idx = idx;
                                break;
                            }
                        }
                    }

                    self.iter.next().unwrap();
                }
                ',' | ';' => {
                    if idx == start_idx {
                        self.iter.next().unwrap();
                        end_idx = if let Some((idx2, _)) = self.iter.clone().next() {
                            idx2
                        } else {
                            self.string.len()
                        };
                    } else {
                        end_idx = idx;
                    }

                    break;
                }
                _ => {
                    self.iter.next().unwrap();
                }
            }
        }

        Some(&self.string[start_idx..end_idx].trim())
    }

    fn position(&self) -> Position {
        let mut pos = Position { line: 1, column: 1 };
        let mut iter = self.string.char_indices();

        while let Some((idx, ch)) = iter.next() {
            if idx >= self.latest_offset {
                break;
            }

            match ch {
                // CR
                '\r' => {
                    if let Some((_, ch2)) = iter.clone().next() {
                        // LF
                        if ch2 == '\n' {
                            iter.next().unwrap();
                        }
                    }
                    pos.line += 1;
                    pos.column = 1;
                }
                // LF
                '\n' => {
                    pos.line += 1;
                    pos.column = 1;
                }
                _ => {
                    pos.column += 1;
                }
            }
        }

        pos
    }
}
