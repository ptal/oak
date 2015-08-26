// Copyright 2015 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use parse_state::ParseState;
use parse_success::ParseSuccess;
use ConsumePrefix;

#[inline]
pub fn parse_any_single_char<S>(mut stream: S) -> ParseState<S, char> where
 S: Iterator<Item=char> + Clone
{
  match stream.next() {
    Some(any) => ParseState::success(stream, any),
    None => ParseState::error(stream, "<character>")
  }
}

#[inline]
pub fn recognize_any_single_char<S>(mut stream: S) -> ParseState<S, ()> where
 S: Iterator<Item=char> + Clone
{
  match stream.next() {
    Some(_) => ParseState::stateless(stream),
    None => ParseState::error(stream, "<character>")
  }
}

#[inline]
pub fn parse_match_literal<S>(mut stream: S, lit: &'static str)
  -> ParseState<S, ()> where
 S: Iterator + Clone + ConsumePrefix<&'static str>
{
  let past_stream = stream.clone();
  if stream.consume_prefix(lit) {
    ParseState::stateless(stream)
  } else {
    ParseState::error(past_stream, lit)
  }
}

#[inline]
pub fn recognize_match_literal<S>(stream: S, lit: &'static str)
  -> ParseState<S, ()> where
 S: Iterator + Clone + ConsumePrefix<&'static str>
{
  parse_match_literal(stream, lit)
}

#[inline]
/// We erase the errors generated inside a `!e` expression because it is hard to correctly use (see paper Maidl & al. 2014 on error reporting).
pub fn not_predicate<S>(state: ParseState<S, ()>, stream: S)
  -> ParseState<S, ()> where
 S: Clone
{
  match state.success {
    Some(_) => ParseState::empty_error(stream),
    _ => ParseState::stateless(stream)
  }
}

#[inline]
pub fn and_predicate<S>(state: ParseState<S, ()>, stream: S)
  -> ParseState<S, ()>
{
  state.map(|_| ParseSuccess::stateless(stream))
}

#[inline]
pub fn optional_recognizer<S>(state: ParseState<S, ()>, stream: S)
  -> ParseState<S, ()>
{
  state.or_stateless(stream)
}

#[inline]
pub fn optional_parser<S, T>(state: ParseState<S, T>, stream: S)
  -> ParseState<S, Option<T>>
{
  state.map_or_else(
    || ParseSuccess::new(stream, None),
    |success| success.map(|data| Some(data)))
}
