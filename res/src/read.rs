// SPDX-License-Identifier: MPL-2.0

use std::{cell::RefCell, iter::Peekable, str::CharIndices};

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
    fn get_symbol(&self) -> Option<&'a str>;
    fn next_symbol(&self) -> Option<&'a str>;
    fn position(&self) -> Position;
}

pub struct StrRead<'a> {
    string: &'a str,
    state: RefCell<StrReadState<'a>>,
}

pub struct StrReadState<'a> {
    iter: Peekable<CharIndices<'a>>,
    latest_offset: usize,
}

impl<'a> StrRead<'a> {
    pub fn new(string: &'a str) -> Self {
        Self {
            string,
            state: RefCell::new(StrReadState {
                iter: string.char_indices().peekable(),
                latest_offset: 0,
            }),
        }
    }

    fn skip_until_not_whitespace(state: &mut StrReadState) -> Option<()> {
        while let Some((_, ch)) = state.iter.peek() {
            match ch {
                ' ' | '\t' | '\r' | '\n' => {
                    state.iter.next().unwrap();
                }
                _ => {
                    return Some(());
                }
            }
        }

        None
    }

    fn skip_until_not_comment(state: &mut StrReadState) -> Option<()> {
        while let Some((_, ch)) = state.iter.next() {
            if ch == '*' {
                if let Some((_, '/')) = state.iter.peek() {
                    state.iter.next().unwrap();
                    return Some(());
                }
            }
        }

        None
    }
}

impl<'a> Read<'a> for StrRead<'a> {
    fn get_symbol(&self) -> Option<&'a str> {
        let iter = self.state.borrow().iter.clone();
        let symbol = self.next_symbol();
        self.state.borrow_mut().iter = iter;

        symbol
    }

    fn next_symbol(&self) -> Option<&'a str> {
        let state = &mut *self.state.borrow_mut();

        Self::skip_until_not_whitespace(state)?;
        state.latest_offset = state.iter.peek()?.0;
        let mut start_idx = state.latest_offset;
        let mut end_idx = self.string.len();

        while let Some(&(idx, ch)) = state.iter.peek() {
            match ch {
                '\r' | '\n' => {
                    if idx == start_idx {
                        Self::skip_until_not_whitespace(state)?;

                        state.latest_offset = state.iter.peek()?.0;
                        start_idx = state.latest_offset;
                        continue;
                    } else {
                        end_idx = idx;
                        break;
                    }
                }
                '/' => {
                    if let Some((_, '*')) = state.iter.clone().nth(1) {
                        if idx == start_idx {
                            state.iter.nth(1).unwrap();
                            Self::skip_until_not_comment(state)?;

                            state.latest_offset = state.iter.peek()?.0;
                            start_idx = state.latest_offset;
                            continue;
                        } else {
                            end_idx = idx;
                            break;
                        }
                    }

                    state.iter.next().unwrap();
                }
                ',' | ';' => {
                    if idx == start_idx {
                        state.iter.next().unwrap();
                        end_idx = if let Some(&(idx2, _)) = state.iter.peek() {
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
                    state.iter.next().unwrap();
                }
            }
        }

        Some(self.string[start_idx..end_idx].trim())
    }

    fn position(&self) -> Position {
        let state = &*self.state.borrow();

        let mut pos = Position { line: 1, column: 1 };
        let mut iter = self.string.char_indices().peekable();

        while let Some((idx, ch)) = iter.next() {
            if idx >= state.latest_offset {
                break;
            }

            match ch {
                // CR
                '\r' => {
                    iter.next_if(|&(_, ch2)| ch2 == '\n');

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
