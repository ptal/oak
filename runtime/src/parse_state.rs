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

//! A parsing state indicates the current status of the parsing. It is mainly used by compiled PEG combinators.

use stream::*;
use self::ParseResult::*;
use std::collections::hash_set::HashSet;
use std::cmp::Ord;
use std::fmt::{Formatter, Debug, Error};

pub trait IntoState<S, T>
{
  fn into_state(self) -> ParseState<S, T>;
}

impl<S, T, R> IntoState<S, T> for R where
  R: Stream<Output=S>,
  S: Ord + Clone + HasNext
{
  fn into_state(self) -> ParseState<S, T> {
    ParseState::new(self.stream())
  }
}

pub struct ParseExpectation<S>
{
  expected: HashSet<&'static str>,
  farthest_read: S
}

impl<S> ParseExpectation<S>
{
  pub fn new(farthest_read: S, expected: Vec<&'static str>) -> ParseExpectation<S> {
    ParseExpectation {
      expected: expected.into_iter().collect(),
      farthest_read: farthest_read
    }
  }
}

impl<S> ParseExpectation<S> where
 S: Location + CodeSnippet
{
  pub fn expected_items(&self) -> String {
    let mut desc = String::new();
    if self.expected.len() > 0 {
      for expect in &self.expected {
        desc.push('`');
        desc.push_str(expect);
        desc.push_str("` or ");
      }
      let len_without_last_or = desc.len() - 4;
      desc.truncate(len_without_last_or);
    }
    desc
  }
}

/// Prints an error message of the form: ```1:1: unexpected `a+1`, expecting `(` or `["0-9"]`.``` where `1:1` is the line and the column where the error occurred.
impl<S> Debug for ParseExpectation<S> where
 S: Location + CodeSnippet
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    let location = self.farthest_read.location();
    let expected = self.expected_items();
    let snippet = self.farthest_read.code_snippet(10usize);
    formatter.write_fmt(
      format_args!("{}: unexpected `{}`, expecting {}.", location, snippet, expected))
  }
}

pub enum ParseResult<S, T>
{
  Success(T),
  Partial(T, ParseExpectation<S>),
  Failure(ParseExpectation<S>)
}

impl<S, T> Debug for ParseResult<S, T> where
 T: Debug,
 S: HasNext + Location + CodeSnippet
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    match self {
      &Success(ref data) => {
        formatter.write_fmt(format_args!(
          "Full match, got data `{:?}`.", data))
      }
      &Partial(ref data, ref expectation) => {
        try!(formatter.write_fmt(format_args!(
          "Partial match, got data `{:?}`. It stopped because:\n\t",
          data)));
        expectation.fmt(formatter)
      }
      &Failure(ref expectation) => {
        try!(formatter.write_str("Error:\n\t"));
        expectation.fmt(formatter)
      }
    }
  }
}

/// `ParseState<S, T>` reads value from the stream `S` and build an AST of type `T`.
/// Error strategy: Even in case of success, we keep error information in case we fail later. Think about parsing "abaa" with `"ab"* "c"`, it will directly fails on `"c"`, so it is better to report an error such as `expected "ab" but got "aa"` since the input partially matches "ab"`.
pub struct ParseState<S, T>
{
  /// The farthest read into the stream at which we encountered an error.
  pub farthest_read: S,
  /// Expected items at position `farthest_read`. Duplicate entries are possible.
  pub expected: Vec<&'static str>,
  pub failed: bool,
  /// The current stream that can be partially or fully consumed.
  pub current: S,
  /// Contains the AST if the current state is successful and `None` if it is erroneous.
  pub data: Option<T>
}

impl<S, T> ParseState<S, T> where
 S: Ord + Clone + HasNext
{
  #[inline]
  pub fn new(stream: S) -> ParseState<S, T> {
    ParseState {
      farthest_read: stream.clone(),
      expected: vec![],
      failed: false,
      current: stream,
      data: None
    }
  }

  pub fn is_failed(&self) -> bool {
    self.failed
  }

  pub fn is_successful(&self) -> bool {
    !self.is_failed()
  }

  #[inline]
  pub fn error(&mut self, expect: &'static str) {
    self.failed = true;
    if self.current > self.farthest_read {
      self.farthest_read = self.current.clone();
      self.expected = vec![expect];
    }
    else if self.current == self.farthest_read {
      self.expected.push(expect);
    }
  }

  // TODO: find a way to specialize success when U = T.
  #[inline]
  pub fn success<U>(self, data: U) -> ParseState<S, U> {
    ParseState {
      farthest_read: self.farthest_read,
      expected: self.expected,
      failed: false,
      current: self.current,
      data: Some(data)
    }
  }

  pub fn failure<U>(self) -> ParseState<S, U> {
    ParseState {
      farthest_read: self.farthest_read,
      expected: self.expected,
      failed: true,
      current: self.current,
      data: None
    }
  }

  pub fn mark(&self) -> S {
    assert!(!self.failed, "Marking a failed ParseState is not allowed.");
    self.current.clone()
  }

  pub fn restore_from_failure(self, mark: S) -> ParseState<S, ()> {
    assert!(self.failed, "Restoring a successful ParseState is not allowed.");
    self.restore(mark)
  }

  pub fn restore(self, mark: S) -> ParseState<S, ()> {
    assert!(self.data.is_none(), "Restoring a ParseState with data is not allowed.");
    ParseState {
      farthest_read: self.farthest_read,
      expected: self.expected,
      failed: false,
      current: mark,
      data: None
    }
  }

  /// Transforms `self` into a more usable `ParseResult` value. It is useful when the state is terminal or if the state will not be further transformed.
  pub fn into_result(self) -> ParseResult<S, T> {
    let expectation = ParseExpectation::new(self.farthest_read, self.expected);
    match self.data {
      Some(data) => {
        if self.current.has_next() {
          Partial(data, expectation)
        }
        else {
          Success(data)
        }
      }
      None => {
        assert!(self.failed, "Failure status must be true when extracting a failed result.");
        Failure(expectation)
      }
    }
  }

  pub fn extract_data(self) -> (ParseState<S, ()>, T) {
    assert!(self.is_successful() && self.data.is_some(),
      "Data extraction is only possible if the state is successful and contains data.");
    let data = self.data.unwrap();
    let state = ParseState {
      farthest_read: self.farthest_read,
      expected: self.expected,
      failed: self.failed,
      current: self.current,
      data: None
    };
    (state, data)
  }

  pub fn unwrap_data(self) -> T {
    self.data.expect("data in ParseState (unwrap_data)")
  }
}

impl<S> ParseState<S, ()>
{
  // This is specific to recognizer where unit data does not need to be extracted. We also want to preserve the "no-data" precondition of `restore`.
  pub fn discard_data(&mut self) {
    self.data = None;
  }
}

impl<S, T, I> Iterator for ParseState<S, T> where
 S: Iterator<Item=I>
{
  type Item = I;
  fn next(&mut self) -> Option<Self::Item> {
    self.current.next()
  }
}

impl<S, T, P> ConsumePrefix<P> for ParseState<S, T> where
  S: ConsumePrefix<P>
{
  fn consume_prefix(&mut self, prefix: P) -> bool {
    self.current.consume_prefix(prefix)
  }
}
