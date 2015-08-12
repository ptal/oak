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

pub type ParseResult<T> = Result<ParseState<T>, String>;

pub struct ParseState<T>
{
  pub data: T,
  pub offset: usize
}

impl<T> ParseState<T>
{
  #[inline]
  pub fn new(data: T, offset: usize) -> ParseState<T>
  {
    ParseState {
      data: data,
      offset: offset
    }
  }
}

impl ParseState<()>
{
  #[inline]
  pub fn stateless(offset: usize) -> ParseState<()>
  {
    ParseState::new((), offset)
  }

  #[inline]
  pub fn erase<T>(source: ParseState<T>) -> ParseState<()>
  {
    ParseState {
      data: (),
      offset: source.offset
    }
  }

  pub fn full_read(&self, input: &str) -> bool
  {
    debug_assert!(self.offset <= input.len());
    self.offset == input.len()
  }

  pub fn partial_read(&self, input: &str) -> bool
  {
    !self.full_read(input)
  }
}

#[inline]
pub fn parse_any_single_char(input: &str, offset: usize) -> ParseResult<char>
{
  if offset < input.len() {
    let any = input.char_at(offset);
    Ok(ParseState::new(any, offset + any.len_utf8()))
  } else {
    Err(format!("End of input when matching `.`"))
  }
}

#[inline]
pub fn recognize_any_single_char(input: &str, offset: usize) -> ParseResult<()>
{
  parse_any_single_char(input, offset).map(|state| ParseState::erase(state))
}

#[inline]
pub fn parse_match_literal(input: &str, offset: usize, lit: &str, lit_len: usize)
  -> ParseResult<()>
{
  if offset >= input.len() {
    Err(format!("End of input when matching the literal `{}`", lit))
  } else if input[offset..].starts_with(lit) {
    Ok(ParseState::stateless(offset + lit_len))
  } else {
    Err(format!("Expected `{}` but got `{}`", lit, &input[offset..]))
  }
}

#[inline]
pub fn recognize_match_literal(input: &str, offset: usize, lit: &str, lit_len: usize)
  -> ParseResult<()>
{
  parse_match_literal(input, offset, lit, lit_len)
}

#[inline]
pub fn not_predicate(state: ParseResult<()>, offset: usize)
  -> ParseResult<()>
{
  match state {
    Ok(_) => Err(format!("An `!expr` failed.")),
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
