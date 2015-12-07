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

//! Data carried by a successful parsing state.

use HasNext;

/// Type `ParseSuccess` contains information of a successful parsing state.
#[derive(Debug)]
pub struct ParseSuccess<S, T>
{
  /// The current stream that can be partially or fully consumed.
  pub stream: S,
  /// AST built from items read in `stream` that are before the current state of `stream`. It does not necessarily contains `data` built from the beginning of `stream` since it depends on the state in which `stream` was before starting the parsing.
  pub data: T
}

impl<S, T> ParseSuccess<S, T>
{
  #[inline]
  pub fn new(stream: S, data: T) -> ParseSuccess<S, T> {
    ParseSuccess {
      stream: stream,
      data: data
    }
  }

  /// Maps `op` to the current `data` while keeping `stream` unchanged.
  #[inline]
  pub fn map<U, F>(self, op: F) -> ParseSuccess<S, U> where
   F: FnOnce(T) -> U
  {
    ParseSuccess {
      stream: self.stream,
      data: op(self.data)
    }
  }
}

impl<S, T> ParseSuccess<S, T> where
 S: HasNext
{
  /// Returns `true` if `stream` is entirely consumed.
  pub fn full_read(&self) -> bool {
    !self.stream.has_next()
  }

  /// Returns `true` if `stream` still contains at least one item.
  pub fn partial_read(&self) -> bool {
    !self.full_read()
  }
}

impl<S> ParseSuccess<S, ()>
{
  #[inline]
  pub fn stateless(stream: S) -> ParseSuccess<S, ()> {
    ParseSuccess::new(stream, ())
  }
}
