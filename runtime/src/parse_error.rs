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

//! Data carried by a parsing state even if the parsing state is currently successful.

use {Location, CodeSnippet};
use std::collections::hash_set::HashSet;
use std::cmp::Ord;
use std::fmt::{Formatter, Display, Error};

/// Type `ParseError` contains the list of all expected items that could have been parsed at position `farthest_read`.
#[derive(Clone, Debug)]
pub struct ParseError<S>
{
  /// The farthest read into the stream at which we encountered an error.
  pub farthest_read: S,
  /// Expected items at position `farthest_read`. Duplicate entries are possible.
  pub expected: Vec<&'static str>
}

impl<S> ParseError<S>
{
  pub fn unique(farthest_read: S, expect: &'static str) -> ParseError<S> {
    ParseError {
      farthest_read: farthest_read,
      expected: vec![expect]
    }
  }

  pub fn empty(farthest_read: S) -> ParseError<S> {
    ParseError {
      farthest_read: farthest_read,
      expected: vec![]
    }
  }

  pub fn expected_items(&self) -> String {
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

impl<S> ParseError<S> where
 S: Ord
{
  /// Merge two parsing errors. We only keep information of the error that occurred the farthest in the stream. In case of equality, both expected item lists are merged into a new one. It does not remove identical expected items.
  pub fn merge(mut self, other: ParseError<S>) -> ParseError<S> {
    if self.farthest_read > other.farthest_read {
      self
    }
    else if self.farthest_read < other.farthest_read {
      other
    }
    else {
      self.expected.extend(other.expected.into_iter());
      self
    }
  }
}

/// Prints an error message of the form: ```1:1: unexpected `a+1`, expecting `(` or `["0-9"]`.``` where `1:1` is the line and the column where the error occurred.
impl<S> Display for ParseError<S> where
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


#[cfg(test)]
mod test {
  use super::*;
  use stream::*;

  #[test]
  fn test_error_merge() {
    let mut s1 = "abc".stream();
    let s2 = s1.clone();
    s1.next();

    let err1 = ParseError::unique(s1, "err1");
    let err2 = ParseError::unique(s2, "err2");
    let err1_2_merge = err1.clone().merge(err2.clone());
    assert!(err1_2_merge.farthest_read == err1.farthest_read);
    assert!(err1_2_merge.expected == vec!["err1"]);

    let err2_merge = err2.clone().merge(err2.clone());
    assert!(err2_merge.farthest_read == err2.farthest_read);
    assert!(err2_merge.expected == vec!["err2", "err2"]);

    let err2_1_merge = err2.clone().merge(err1.clone());
    assert!(err2_1_merge.farthest_read == err1.farthest_read);
    assert!(err2_1_merge.expected == vec!["err1"]);
  }
}
