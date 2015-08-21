// Copyright 2014 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![feature(str_char)]

use std::collections::hash_set::HashSet;

pub type ParseResult<T> = Result<ParseState<T>, ParseError>;

#[derive(Clone, Debug)]
pub struct ParseError
{
  pub farthest_offset: usize,
  pub expected: Vec<&'static str>
}

impl ParseError
{
  pub fn unique(offset: usize, expect: &'static str) -> ParseError {
    ParseError {
      farthest_offset: offset,
      expected: vec![expect]
    }
  }

  pub fn empty(offset: usize) -> ParseError {
    ParseError {
      farthest_offset: offset,
      expected: vec![]
    }
  }

  pub fn join(mut self, other: ParseError) -> ParseError {
    if self.farthest_offset > other.farthest_offset {
      self
    }
    else if self.farthest_offset < other.farthest_offset {
      other
    }
    else {
      self.expected.extend(other.expected.into_iter());
      self
    }
  }

  // Partially taken from https://github.com/kevinmehall/rust-peg/blob/master/src/translate.rs
  pub fn line_column(&self, source: &str) -> (usize, usize) {
    let mut remaining = self.farthest_offset;
    let mut line_no = 1usize;
    for line in source.lines_any() {
      let line_len = line.len() + 1;
      if remaining < line_len {
        break;
      }
      remaining -= line_len;
      line_no += 1;
    }
    (line_no, remaining + 1)
  }

  pub fn description(&self, source: &str) -> String {
    let (line, column) = self.line_column(source);
    let expected = self.expected_desc();
    format!("{}:{}: unexpected `{}`, expecting {}.", line, column, self.code_snippet(source), expected)
  }

  fn code_snippet<'a>(&self, source: &'a str) -> &'a str
  {
    let code_snippet_len = 10usize;
    let len = std::cmp::min(source.len() - self.farthest_offset, code_snippet_len);
    &source[self.farthest_offset..len]
  }

  fn expected_desc(&self) -> String {
    let expected: HashSet<&'static str> = self.expected.clone().into_iter().collect();
    let mut desc = String::new();
    for expect in expected {
      desc.push('`');
      desc.push_str(expect);
      desc.push_str("` or ");
    }
    let len_without_last_or = desc.len() - 4;
    desc.truncate(len_without_last_or);
    desc
  }
}

#[derive(Debug)]
pub struct ParseState<T>
{
  pub data: T,
  pub offset: usize
}

impl<T> ParseState<T>
{
  #[inline]
  pub fn new(data: T, offset: usize) -> ParseState<T> {
    ParseState {
      data: data,
      offset: offset
    }
  }
}

impl ParseState<()>
{
  #[inline]
  pub fn stateless(offset: usize) -> ParseState<()> {
    ParseState::new((), offset)
  }

  #[inline]
  pub fn erase<T>(source: ParseState<T>) -> ParseState<()> {
    ParseState {
      data: (),
      offset: source.offset
    }
  }

  pub fn full_read(&self, input: &str) -> bool {
    debug_assert!(self.offset <= input.len());
    self.offset == input.len()
  }

  pub fn partial_read(&self, input: &str) -> bool {
    !self.full_read(input)
  }
}

#[inline]
pub fn parse_any_single_char(input: &str, offset: usize) -> ParseResult<char> {
  if offset < input.len() {
    let any = input.char_at(offset);
    Ok(ParseState::new(any, offset + any.len_utf8()))
  } else {
    Err(ParseError::unique(offset, "<character>"))
  }
}

#[inline]
pub fn recognize_any_single_char(input: &str, offset: usize) -> ParseResult<()> {
  parse_any_single_char(input, offset).map(|state| ParseState::erase(state))
}

#[inline]
pub fn parse_match_literal(input: &str, offset: usize, lit: &'static str, lit_len: usize)
  -> ParseResult<()>
{
  if offset < input.len() && input[offset..].starts_with(lit) {
    Ok(ParseState::stateless(offset + lit_len))
  } else {
    Err(ParseError::unique(offset, lit))
  }
}

#[inline]
pub fn recognize_match_literal(input: &str, offset: usize, lit: &'static str, lit_len: usize)
  -> ParseResult<()>
{
  parse_match_literal(input, offset, lit, lit_len)
}

#[inline]
pub fn not_predicate(state: ParseResult<()>, offset: usize)
  -> ParseResult<()>
{
  match state {
    Ok(_) => Err(ParseError::empty(offset)),
    _ => Ok(ParseState::stateless(offset))
  }
}

#[inline]
pub fn and_predicate(state: ParseResult<()>, offset: usize)
  -> ParseResult<()>
{
  state.map(|_| ParseState::stateless(offset))
}

#[inline]
pub fn optional_recognizer(state: ParseResult<()>, offset: usize)
  -> ParseResult<()>
{
  state.or_else(|_| Ok(ParseState::stateless(offset)))
}

#[inline]
pub fn optional_parser<T>(state: ParseResult<T>, offset: usize)
  -> ParseResult<Option<T>>
{
  match state {
    Ok(state) => Ok(ParseState::new(Some(state.data), state.offset)),
    Err(_) => Ok(ParseState::new(None, offset))
  }
}
