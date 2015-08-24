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

pub mod str_stream;

pub trait Producer
{
  type Stream;
  fn producer(self) -> Self::Stream;
}

pub type ParseResult<T> = Result<ParseSuccess<T>, String>;

pub struct ParseState<T>
{
  /// Even in case of success, we keep error information in case we
  /// fail later.
  pub error: ParseError,
  pub success: Option<ParseSuccess<T>>
}

impl<T> ParseState<T>
{
  #[inline]
  pub fn success(data: T, offset: usize) -> ParseState<T> {
    ParseState {
      error: ParseError::empty(offset),
      success: Some(ParseSuccess::new(data, offset))
    }
  }

  #[inline]
  pub fn from_error(error: ParseError) -> ParseState<T> {
    ParseState {
      error: error,
      success: None
    }
  }

  #[inline]
  pub fn error(offset: usize, expect: &'static str) -> ParseState<T> {
    ParseState::from_error(ParseError::unique(offset, expect))
  }

  #[inline]
  pub fn empty_error(offset: usize) -> ParseState<T> {
    ParseState {
      error: ParseError::empty(offset),
      success: None
    }
  }

  #[inline]
  pub fn map<U, F>(self, op: F) -> ParseState<U> where
   F: FnOnce(ParseSuccess<T>) -> ParseSuccess<U>
  {
    ParseState {
      error: self.error,
      success: self.success.map(op)
    }
  }

  #[inline]
  pub fn and_then<U, F>(self, op: F) -> ParseState<U> where
   F: FnOnce(ParseSuccess<T>) -> ParseState<U>
  {
    match self.success {
      Some(success) => {
        let mut state = op(success);
        state.error = state.error.join(self.error);
        state
      }
      None => ParseState::from_error(self.error)
    }
  }

  #[inline]
  pub fn or_else_join<F>(self, op: F) -> ParseState<T> where
   F: FnOnce() -> ParseState<T>
  {
    self.or_else(|err| op().join(err))
  }

  #[inline]
  pub fn join(mut self, error: ParseError) -> ParseState<T> {
    self.error = self.error.join(error);
    self
  }

  #[inline]
  pub fn or_else<F>(self, op: F) -> ParseState<T> where
   F: FnOnce(ParseError) -> ParseState<T>
  {
    match &self.success {
      &Some(_) => self,
      &None => op(self.error)
    }
  }

  #[inline]
  pub fn map_or_else<U, D, F>(self, default: D, f: F) -> ParseState<U> where
   D: FnOnce() -> ParseSuccess<U>,
   F: FnOnce(ParseSuccess<T>) -> ParseSuccess<U>
  {
    ParseState {
      error: self.error,
      success: Some(self.success.map_or_else(default, f))
    }
  }

  #[inline]
  pub fn to_error(mut self) -> ParseState<T> {
    self.success = None;
    self
  }

  pub fn into_result(self, source: &str) -> ParseResult<T> {
    match self.success {
      Some(success) => {
        Ok(success)
      },
      None => {
        Err(self.error.description(source))
      }
    }
  }
}

impl ParseState<()>
{
  #[inline]
  pub fn stateless(offset: usize) -> ParseState<()> {
    ParseState::success((), offset)
  }

  #[inline]
  pub fn to_stateless_success(self, offset: usize) -> ParseState<()> {
    ParseState {
      error: self.error,
      success: self.success.or(Some(ParseSuccess::stateless(offset)))
    }
  }

  #[inline]
  pub fn merge_success(&mut self, other: ParseSuccess<()>)
  {
    debug_assert!(self.success.is_some(), "ParseState::merge_success can only be called if `self` is successfull.");
    self.success = Some(other);
  }
}

impl<T> ParseState<Vec<T>>
{
  #[inline]
  pub fn merge_success(&mut self, other: ParseSuccess<T>)
  {
    debug_assert!(self.success.is_some(), "ParseState::merge_success can only be called if `self` is successfull.");
    let success = self.success.as_mut().unwrap();
    success.data.push(other.data);
    success.offset = other.offset;
  }
}

#[derive(Debug)]
pub struct ParseSuccess<T>
{
  pub data: T,
  pub offset: usize
}

impl<T> ParseSuccess<T>
{
  #[inline]
  pub fn new(data: T, offset: usize) -> ParseSuccess<T> {
    ParseSuccess {
      data: data,
      offset: offset
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

impl ParseSuccess<()>
{
  #[inline]
  pub fn stateless(offset: usize) -> ParseSuccess<()> {
    ParseSuccess::new((), offset)
  }
}

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
    debug_assert!(self.farthest_offset <= source.len());
    if self.farthest_offset == source.len() {
      &"<end-of-file>"
    }
    else {
      let code_snippet_len = 10usize;
      let len = std::cmp::min(source.len() - self.farthest_offset, code_snippet_len);
      &source[self.farthest_offset..][..len]
    }
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

#[inline]
pub fn parse_any_single_char(input: &str, offset: usize) -> ParseState<char> {
  if offset < input.len() {
    let any = input.char_at(offset);
    ParseState::success(any, offset + any.len_utf8())
  } else {
    ParseState::error(offset, "<character>")
  }
}

#[inline]
pub fn recognize_any_single_char(input: &str, offset: usize) -> ParseState<()> {
  if offset < input.len() {
    let any = input.char_at(offset);
    ParseState::stateless(offset + any.len_utf8())
  } else {
    ParseState::error(offset, "<character>")
  }
}

#[inline]
pub fn parse_match_literal(input: &str, offset: usize, lit: &'static str, lit_len: usize)
  -> ParseState<()>
{
  let end_offset = offset + lit_len;
  if end_offset <= input.len() && &input.as_bytes()[offset..end_offset] == lit.as_bytes() {
    ParseState::stateless(end_offset)
  } else {
    ParseState::error(offset, lit)
  }
}

#[inline]
pub fn recognize_match_literal(input: &str, offset: usize, lit: &'static str, lit_len: usize)
  -> ParseState<()>
{
  parse_match_literal(input, offset, lit, lit_len)
}

#[inline]
/// We erase the errors generated inside a `!e` expression because it is hard to correctly use (see paper Maidl & al. 2014 on error reporting).
pub fn not_predicate(state: ParseState<()>, offset: usize)
  -> ParseState<()>
{
  match state.success {
    Some(_) => ParseState::empty_error(offset),
    _ => ParseState::stateless(offset)
  }
}

#[inline]
pub fn and_predicate(state: ParseState<()>, offset: usize)
  -> ParseState<()>
{
  state.map(|_| ParseSuccess::stateless(offset))
}

#[inline]
pub fn optional_recognizer(state: ParseState<()>, offset: usize)
  -> ParseState<()>
{
  state.to_stateless_success(offset)
}

#[inline]
pub fn optional_parser<T>(state: ParseState<T>, offset: usize)
  -> ParseState<Option<T>>
{
  state.map_or_else(
    || ParseSuccess::new(None, offset),
    |success| ParseSuccess::new(Some(success.data), success.offset))
}
