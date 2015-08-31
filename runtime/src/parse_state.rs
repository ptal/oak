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

use std::cmp::Ord;
use stream::HasNext;
use parse_error::ParseError;
use parse_success::ParseSuccess;

pub type ParseResult<S, T> = Result<ParseSuccess<S, T>, ParseError<S>>;

pub struct ParseState<S, T>
{
  /// Even in case of success, we keep error information in case we
  /// fail later. Think about parsing "abaa" with `"ab"* r2`, if `r2`
  /// directly fails, it is better to report an error such as:
  /// `expected "ab" but got "aa"` since the input partially matches "ab".
  pub error: ParseError<S>,
  pub success: Option<ParseSuccess<S, T>>
}

impl<S, T> ParseState<S, T> where
 S: Clone
{
  #[inline]
  pub fn success(stream: S, data: T) -> ParseState<S, T> {
    ParseState {
      error: ParseError::empty(stream.clone()),
      success: Some(ParseSuccess::new(stream, data))
    }
  }

  #[inline]
  pub fn stream(&self) -> S {
    self.assert_success("stream");
    self.success.as_ref().unwrap().stream.clone()
  }
}

impl<S, T> ParseState<S, T>
{
  #[inline]
  pub fn from_error(error: ParseError<S>) -> ParseState<S, T> {
    ParseState {
      error: error,
      success: None
    }
  }

  #[inline]
  pub fn error(stream: S, expect: &'static str) -> ParseState<S, T> {
    ParseState::from_error(ParseError::unique(stream, expect))
  }

  #[inline]
  pub fn empty_error(stream: S) -> ParseState<S, T> {
    ParseState {
      error: ParseError::empty(stream),
      success: None
    }
  }

  #[inline]
  pub fn map<U, F>(self, op: F) -> ParseState<S, U> where
   F: FnOnce(ParseSuccess<S, T>) -> ParseSuccess<S, U>
  {
    ParseState {
      error: self.error,
      success: self.success.map(op)
    }
  }

  #[inline]
  pub fn map_data<U, F>(self, op: F) -> ParseState<S, U> where
   F: FnOnce(T) -> U
  {
    ParseState {
      error: self.error,
      success: self.success.map(|success| success.map(op))
    }
  }

  #[inline]
  pub fn or_else<F>(self, op: F) -> ParseState<S, T> where
   F: FnOnce(ParseError<S>) -> ParseState<S, T>
  {
    match &self.success {
      &Some(_) => self,
      &None => op(self.error)
    }
  }

  #[inline]
  pub fn map_or_else<U, D, F>(self, default: D, f: F) -> ParseState<S, U> where
   D: FnOnce() -> ParseSuccess<S, U>,
   F: FnOnce(ParseSuccess<S, T>) -> ParseSuccess<S, U>
  {
    ParseState {
      error: self.error,
      success: Some(self.success.map_or_else(default, f))
    }
  }

  #[inline]
  pub fn to_error(mut self) -> ParseState<S, T> {
    self.success = None;
    self
  }

  pub fn into_result(self) -> ParseResult<S, T> {
    match self.success {
      Some(success) => {
        Ok(success)
      },
      None => {
        Err(self.error)
      }
    }
  }

  pub fn is_successful(&self) -> bool {
    self.success.is_some()
  }

  fn assert_success(&self, from: &str) {
    debug_assert!(self.is_successful(),
      "`{}`: `self` is required to be in a successful state.", from);
  }
}

impl<S, T> ParseState<S, T> where
 S: HasNext
{
  #[inline]
  pub fn has_successor(&self) -> bool {
    self.success.as_ref().map_or(false, |success| success.stream.has_next())
  }
}

impl<S, T> ParseState<S, T> where
 S: Eq
{
  pub fn stream_eq(&self, other: &S) -> bool {
    self.success.as_ref().map_or(false, |success| &success.stream == other)
  }
}

impl<S, T> ParseState<S, T> where
 S: Ord
{
  #[inline]
  pub fn and_then<U, F>(self, op: F) -> ParseState<S, U> where
   F: FnOnce(ParseSuccess<S, T>) -> ParseState<S, U>
  {
    match self.success {
      Some(success) => {
        op(success).join(self.error)
      }
      None => ParseState::from_error(self.error)
    }
  }

  #[inline]
  pub fn or_else_join<F>(self, op: F) -> ParseState<S, T> where
   F: FnOnce() -> ParseState<S, T>
  {
    self.or_else(|err| op().join(err))
  }

  #[inline]
  pub fn join(mut self, error: ParseError<S>) -> ParseState<S, T> {
    self.error = self.error.join(error);
    self
  }
}

impl<S> ParseState<S, ()> where
 S: Clone
{
  #[inline]
  pub fn stateless(stream: S) -> ParseState<S, ()> {
    ParseState::success(stream, ())
  }
}

impl<S> ParseState<S, ()>
{
  #[inline]
  pub fn or_stateless(self, stream: S) -> ParseState<S, ()> {
    ParseState {
      error: self.error,
      success: self.success.or(Some(ParseSuccess::stateless(stream)))
    }
  }

  #[inline]
  pub fn merge_success(&mut self, other: ParseSuccess<S, ()>)
  {
    self.assert_success("ParseState<S, ()>::merge_success");
    self.success = Some(other);
  }
}

impl<S, T> ParseState<S, Vec<T>>
{
  #[inline]
  pub fn merge_success(&mut self, other: ParseSuccess<S, T>)
  {
    self.assert_success("ParseState<S, Vec<T>>::merge_success");
    let success = self.success.as_mut().unwrap();
    success.data.push(other.data);
    success.stream = other.stream;
  }
}
