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

//! A parsing state indicates the current status of the parsing. It is mainly used by PEG combinators.

use std::cmp::Ord;
use stream::HasNext;
use parse_error::ParseError;
use parse_success::ParseSuccess;
use ParseResult;

pub struct ParseState<S, T>
{
  /// Even in case of success, we keep error information in case we
  /// fail later. Think about parsing "abaa" with `"ab"* "c"`, it will directly fails on `"c"`,
  /// so it is better to report an error such as:
  /// `expected "ab" but got "aa"` since the input partially matches "ab".
  pub error: ParseError<S>,
  /// Contains a value if the current state is successful and `None` if it is erroneous.
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

  /// Retrieve the stream from a successful state.
  /// Panics if `self` is not successful.
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

  /// Maps `op` to the success value if the state is successful. It does not alter the errors list.
  #[inline]
  pub fn map<U, F>(self, op: F) -> ParseState<S, U> where
   F: FnOnce(ParseSuccess<S, T>) -> ParseSuccess<S, U>
  {
    ParseState {
      error: self.error,
      success: self.success.map(op)
    }
  }

  /// Maps `op` to the data (AST) if the state is successful. It does not alter the errors list.
  #[inline]
  pub fn map_data<U, F>(self, op: F) -> ParseState<S, U> where
   F: FnOnce(T) -> U
  {
    ParseState {
      error: self.error,
      success: self.success.map(|success| success.map(op))
    }
  }

  /// Calls `op` if the state is not successful, otherwise returns the `self` unchanged.
  #[inline]
  pub fn or_else<F>(self, op: F) -> ParseState<S, T> where
   F: FnOnce(ParseError<S>) -> ParseState<S, T>
  {
    match &self.success {
      &Some(_) => self,
      &None => op(self.error)
    }
  }

  /// Applies a function to the contained value (if `self` is successful), or computes a default (if not).
  /// The state returns is always successful. The errors list is unchanged.
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

  /// Transforms `self` into an erroneous state by erasing successful information (if any). The errors list is unchanged.
  #[inline]
  pub fn to_error(mut self) -> ParseState<S, T> {
    self.success = None;
    self
  }

  /// Transforms `self` into a more usable `ParseResult` value. It is useful when the state is terminal or if the state will not be further transformed.
  pub fn into_result(self) -> ParseResult<S, T> {
    match self.success {
      Some(success) => {
        Ok((success, self.error))
      },
      None => {
        Err(self.error)
      }
    }
  }

  /// Extract the underlying data (AST) from the current state.
  /// Panics if the state is not successful.
  #[inline]
  pub fn unwrap_data(self) -> T {
    self.assert_success("ParseState::unwrap_data");
    self.success.unwrap().data
  }

  /// Returns `true` if the state is successful, otherwise returns `false`.
  #[inline]
  pub fn is_successful(&self) -> bool {
    self.success.is_some()
  }

  #[inline(always)]
  fn assert_success(&self, from: &str) {
    debug_assert!(self.is_successful(),
      "`{}`: `self` is required to be in a successful state.", from);
  }
}

impl<S, T> ParseState<S, T> where
 S: HasNext
{
  /// Returns `true` if the state has a successor, otherwise returns `false`.
  /// Erroneous states or states with a consumed stream are terminals and cannot have any successors.
  #[inline]
  pub fn has_successor(&self) -> bool {
    self.success.as_ref().map_or(false, |success| success.stream.has_next())
  }
}

impl<S, T> ParseState<S, T> where
 S: Eq
{
  /// Returns `false` if `self` is erroneous or if the current stream is not equal to `other`.
  pub fn stream_eq(&self, other: &S) -> bool {
    self.success.as_ref().map_or(false, |success| &success.stream == other)
  }
}

impl<S, T> ParseState<S, T> where
 S: Ord
{
  /// Calls `op` if the state is successful and merges both error lists. Otherwise returns an erroneous state with the same errors list as `self`.
  #[inline]
  pub fn and_then<U, F>(self, op: F) -> ParseState<S, U> where
   F: FnOnce(ParseSuccess<S, T>) -> ParseState<S, U>
  {
    match self.success {
      Some(success) => {
        op(success).merge_error(self.error)
      }
      None => ParseState::from_error(self.error)
    }
  }

  /// Calls `op` if `self` is erroneous and merges both error lists. Otherwise returns `self`.
  #[inline]
  pub fn or_else_merge<F>(self, op: F) -> ParseState<S, T> where
   F: FnOnce() -> ParseState<S, T>
  {
    self.or_else(|err| op().merge_error(err))
  }

  /// Merge error lists of `self` and `error`. It does not remove duplicate entries.
  #[inline]
  pub fn merge_error(mut self, error: ParseError<S>) -> ParseState<S, T> {
    self.error = self.error.merge(error);
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
