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
    fn peek_sym(&self) -> Option<&'a str>;
    fn next_sym(&self) -> Option<&'a str>;
    fn position(&self) -> Position;
}

pub struct StrRead<'a> {
    string: &'a str,
    state: RefCell<StrReadState<'a>>,
}

#[derive(Clone)]
pub struct StrReadState<'a> {
    iter: Peekable<CharIndices<'a>>,
    prev_symbol: &'a str,
    latest_offset: usize,
}

impl<'a> StrRead<'a> {
    pub fn new(string: &'a str) -> Self {
        Self {
            string,
            state: RefCell::new(StrReadState {
                iter: string.char_indices().peekable(),
                prev_symbol: "",
                latest_offset: 0,
            }),
        }
    }

    fn skip_until_not_whitespace(iter: &mut Peekable<CharIndices<'a>>) -> Option<()> {
        while let Some((_, ch)) = iter.peek() {
            match ch {
                ' ' | '\t' | '\r' | '\n' => {
                    iter.next().unwrap();
                }
                _ => {
                    return Some(());
                }
            }
        }

        None
    }

    fn skip_until_not_comment(iter: &mut Peekable<CharIndices<'a>>) -> Option<()> {
        while let Some((_, ch)) = iter.next() {
            if ch == '*' {
                if let Some((_, '/')) = iter.peek() {
                    iter.next().unwrap();
                    return Some(());
                }
            }
        }

        None
    }
}

impl<'a> Read<'a> for StrRead<'a> {
    fn peek_sym(&self) -> Option<&'a str> {
        let prev_state = self.state.borrow().clone();
        let symbol = self.next_sym();
        let mut state = self.state.borrow_mut();

        state.iter = prev_state.iter;
        state.prev_symbol = prev_state.prev_symbol;

        symbol
    }

    fn next_sym(&self) -> Option<&'a str> {
        let mut iter = self.state.borrow().iter.clone();

        Self::skip_until_not_whitespace(&mut iter)?;

        let mut begin_idx = iter.peek()?.0;
        let mut end_idx = self.string.len();
        let mut prev_whitespace = false;

        while let Some(&(idx, ch)) = iter.peek() {
            match ch {
                '\r' | '\n' => {
                    if idx == begin_idx {
                        Self::skip_until_not_whitespace(&mut iter)?;
                        begin_idx = iter.peek()?.0;
                        continue;
                    }

                    if !prev_whitespace {
                        end_idx = idx;
                    }

                    break;
                }
                '/' => {
                    if let Some((_, '*')) = iter.clone().nth(1) {
                        if idx == begin_idx {
                            iter.nth(1).unwrap();
                            Self::skip_until_not_comment(&mut iter)?;
                            begin_idx = iter.peek()?.0;
                            continue;
                        }

                        if !prev_whitespace {
                            end_idx = idx;
                        }

                        break;
                    }

                    iter.next().unwrap();
                }
                ',' | ';' => {
                    if idx == begin_idx {
                        iter.next().unwrap();
                        end_idx = if let Some(&(idx, _)) = iter.peek() {
                            idx
                        } else {
                            self.string.len()
                        };
                    } else if !prev_whitespace {
                        end_idx = idx;
                    }

                    break;
                }
                ' ' | '\t' => {
                    if !prev_whitespace {
                        prev_whitespace = true;
                        end_idx = idx;
                    }

                    iter.next().unwrap();
                }
                _ => {
                    prev_whitespace = false;
                    iter.next().unwrap();
                }
            }
        }

        let state = &mut *self.state.borrow_mut();
        let mut symbol = &self.string[begin_idx..end_idx];

        debug_assert!(!symbol.ends_with(|c| matches!(c, ' ' | '\t')));

        if matches!(symbol, "," | ";") && matches!(state.prev_symbol, "," | ";") {
            symbol = "";
        } else {
            state.iter = iter;
            state.latest_offset = begin_idx;
        }

        state.prev_symbol = symbol;

        Some(symbol)
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
                '\r' => {
                    iter.next_if(|&(_, ch)| ch == '\n');

                    pos.line += 1;
                    pos.column = 1;
                }
                '\n' => {
                    pos.line += 1;
                    pos.column = 1;
                }
                '\t' => {
                    pos.column += 4 - (pos.column - 1) % 4;
                }
                _ => {
                    pos.column += 1;
                }
            }
        }

        pos
    }
}
