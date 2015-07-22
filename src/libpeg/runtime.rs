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

pub trait Parser
{
  fn parse<'a>(&self, input: &'a str) -> Result<Option<&'a str>, String>;
}

pub struct ParseState<T>
{
  pub data: T,
  pub offset: usize
}

impl<T> ParseState<T>
{
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
  pub fn stateless(offset: usize) -> ParseState<()>
  {
    ParseState::new((), offset)
  }

  pub fn erase<T>(source: ParseState<T>) -> ParseState<()>
  {
    ParseState {
      data: (),
      offset: source.offset
    }
  }
}

pub fn parse_any_single_char(input: &str, offset: usize) -> Result<ParseState<char>, String>
{
  if offset < input.len() {
    let any = input.char_at(offset);
    Ok(ParseState::new(any, offset + any.len_utf8()))
  } else {
    Err(format!("End of input when matching `.`"))
  }
}

pub fn recognize_any_single_char(input: &str, offset: usize) -> Result<ParseState<()>, String>
{
  parse_any_single_char(input, offset).map(|state| ParseState::erase(state))
}

pub fn parse_match_literal(input: &str, offset: usize, lit: &str, lit_len: usize)
  -> Result<ParseState<()>, String>
{
  if offset >= input.len() {
    Err(format!("End of input when matching the literal `{}`", lit))
  } else if input[offset..].starts_with(lit) {
    Ok(ParseState::stateless(offset + lit_len))
  } else {
    Err(format!("Expected `{}` but got `{}`", lit, &input[offset..]))
  }
}

pub fn recognize_match_literal(input: &str, offset: usize, lit: &str, lit_len: usize)
  -> Result<ParseState<()>, String>
{
  parse_match_literal(input, offset, lit, lit_len)
}

pub fn make_result<'a, T>(input: &'a str, parsing_res: &Result<ParseState<T>, String>)
 -> Result<Option<&'a str>, String>
{
  match parsing_res {
    &Ok(ref state) => {
      assert!(state.offset <= input.len());
      if state.offset == input.len() {
        Ok(None)
      } else {
        Ok(Some(&input[state.offset..]))
      }
    },
    &Err(ref msg) => Err(msg.clone())
  }
}
