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

  pub fn restore(&mut self, mark: S) {
    assert!(self.failed, "Restoring a successful ParseState is not allowed.");
    self.failed = false;
    self.current = mark;
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

// impl<S, T> ParseState<S, T>
// {
//   /// Maps `op` to the success value if the state is successful. It does not alter the errors list.
//   #[inline]
//   pub fn map<U, F>(self, op: F) -> ParseState<S, U> where
//    F: FnOnce(ParseSuccess<S, T>) -> ParseSuccess<S, U>
//   {
//     ParseState {
//       error: self.error,
//       success: self.success.map(op)
//     }
//   }

//   /// Maps `op` to the data (AST) if the state is successful. It does not alter the errors list.
//   #[inline]
//   pub fn map_data<U, F>(self, op: F) -> ParseState<S, U> where
//    F: FnOnce(T) -> U
//   {
//     ParseState {
//       error: self.error,
//       success: self.success.map(|success| success.map(op))
//     }
//   }

//   /// Calls `op` if the state is not successful, otherwise returns the `self` unchanged.
//   #[inline]
//   pub fn or_else<F>(self, op: F) -> ParseState<S, T> where
//    F: FnOnce(ParseError<S>) -> ParseState<S, T>
//   {
//     match &self.success {
//       &Some(_) => self,
//       &None => op(self.error)
//     }
//   }

//   /// Applies a function to the contained value (if `self` is successful), or computes a default (if not).
//   /// The state returns is always successful. The errors list is unchanged.
//   #[inline]
//   pub fn map_or_else<U, D, F>(self, default: D, f: F) -> ParseState<S, U> where
//    D: FnOnce() -> ParseSuccess<S, U>,
//    F: FnOnce(ParseSuccess<S, T>) -> ParseSuccess<S, U>
//   {
//     ParseState {
//       error: self.error,
//       success: Some(self.success.map_or_else(default, f))
//     }
//   }

//   /// Transforms `self` into an erroneous state by erasing successful information (if any). The errors list is unchanged.
//   #[inline]
//   pub fn to_error(mut self) -> ParseState<S, T> {
//     self.success = None;
//     self
//   }

//   /// Extract the underlying data (AST) from the current state.
//   /// Panics if the state is not successful.
//   #[inline]
//   pub fn unwrap_data(self) -> T {
//     self.assert_success("ParseState::unwrap_data");
//     self.success.unwrap().data
//   }

//   /// Returns `true` if the state is successful, otherwise returns `false`.
//   #[inline]
//   pub fn is_successful(&self) -> bool {
//     self.success.is_some()
//   }

//   #[inline(always)]
//   fn assert_success(&self, from: &str) {
//     debug_assert!(self.is_successful(),
//       "`{}`: `self` is required to be in a successful state.", from);
//   }
// }

// impl<S, T> ParseState<S, T> where
//  S: HasNext
// {
//   /// Returns `true` if the state has a successor, otherwise returns `false`.
//   /// Erroneous states or states with a consumed stream are terminals and cannot have any successors.
//   #[inline]
//   pub fn has_successor(&self) -> bool {
//     self.success.as_ref().map_or(false, |success| success.stream.has_next())
//   }
// }

// impl<S, T> ParseState<S, T> where
//  S: Eq
// {
//   /// Returns `false` if `self` is erroneous or if the current stream is not equal to `other`.
//   pub fn stream_eq(&self, other: &S) -> bool {
//     self.success.as_ref().map_or(false, |success| &success.stream == other)
//   }
// }

// impl<S, T> ParseState<S, T> where
//  S: Ord
// {
//   /// Calls `op` if the state is successful and merges both error lists. Otherwise returns an erroneous state with the same errors list as `self`.
//   #[inline]
//   pub fn and_then<U, F>(self, op: F) -> ParseState<S, U> where
//    F: FnOnce(ParseSuccess<S, T>) -> ParseState<S, U>
//   {
//     match self.success {
//       Some(success) => {
//         op(success).merge_error(self.error)
//       }
//       None => ParseState::from_error(self.error)
//     }
//   }

//   /// Calls `op` if `self` is erroneous and merges both error lists. Otherwise returns `self`.
//   #[inline]
//   pub fn or_else_merge<F>(self, op: F) -> ParseState<S, T> where
//    F: FnOnce() -> ParseState<S, T>
//   {
//     self.or_else(|err| op().merge_error(err))
//   }

//   /// Merge error lists of `self` and `error`. It does not remove duplicate entries.
//   #[inline]
//   pub fn merge_error(mut self, error: ParseError<S>) -> ParseState<S, T> {
//     self.merge_error_in_place(error);
//     self
//   }

//   #[inline]
//   pub fn merge_error_in_place(&mut self, error: ParseError<S>) {
//     self.error.merge_in_place(error);
//   }
// }

// impl<S> ParseState<S, ()> where
//  S: Clone
// {
//   #[inline]
//   pub fn stateless(stream: S) -> ParseState<S, ()> {
//     ParseState::success(stream, ())
//   }
// }

// impl<S> ParseState<S, ()>
// {
//   #[inline]
//   pub fn or_stateless(self, stream: S) -> ParseState<S, ()> {
//     ParseState {
//       error: self.error,
//       success: self.success.or(Some(ParseSuccess::stateless(stream)))
//     }
//   }
// }

// pub trait MergeSuccess<S, T> {
//   fn merge_success(&mut self, other: ParseSuccess<S, T>);
// }

// impl<S> MergeSuccess<S, ()> for ParseState<S, ()>
// {
//   #[inline]
//   fn merge_success(&mut self, other: ParseSuccess<S, ()>)
//   {
//     self.assert_success("ParseState<S, ()>::merge_success");
//     self.success = Some(other);
//   }
// }

// impl<S, T> MergeSuccess<S, T> for ParseState<S, Vec<T>>
// {
//   #[inline]
//   fn merge_success(&mut self, other: ParseSuccess<S, T>)
//   {
//     self.assert_success("ParseState<S, Vec<T>>::merge_success");
//     let success = self.success.as_mut().unwrap();
//     success.data.push(other.data);
//     success.stream = other.stream;
//   }
// }

// impl<S, T> ParseState<S, T> where
//  S: Ord
// {
//   pub fn soft_merge<U>(&mut self, other: ParseState<S, U>) -> bool where
//     Self: MergeSuccess<S, U>
//   {
//     self.merge_error_in_place(other.error);
//     if let Some(success) = other.success {
//       self.merge_success(success);
//       true
//     }
//     else {
//       false
//     }
//   }
// }
