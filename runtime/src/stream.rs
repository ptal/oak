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

//! Collection of traits for retrieving and manipulating a stream. They are used by implementation of parsing expressions.
//!
//! A stream produces a sequence of items (characters, bytes, etc.) while retaining information on the underlying data traversed. For example, a couple `(File, Iterator<char>)` could represent a stream of characters from a file.

/// Transforms a value into a stream of type `Output`.
pub trait Stream
{
  type Output;
  fn stream(self) -> Self::Output;
}

/// Requirements of a stream of characters. It is currently required by most parser combinators.
pub trait CharStream
 : Clone + Ord + HasNext + Eq
 + Iterator<Item=char>
 + ConsumePrefix<&'static str>
{}

impl<R> CharStream for R where
 R: Clone + Ord + HasNext + Eq
  + Iterator<Item=char>
  + ConsumePrefix<&'static str>
{}

/// Produces a textual representation of the current position in the stream. For example, it can be `2:5` if the position is at line 2 and column 5.
pub trait Location
{
  fn location(&self) -> String;
}

/// Produces a code snippet of size `len_hint` or less starting from the current position in the stream.
pub trait CodeSnippet
{
  fn code_snippet(&self, len_hint: usize) -> String;
}

/// Consumes `prefix` if it fully matches from the current position in the stream. If it does not match, the stream is not altered and `false` is returned.
pub trait ConsumePrefix<P>
{
  fn consume_prefix(&mut self, prefix: P) -> bool;
}

/// Returns `true` if an item can be read from the stream with `Iterator::next`.
pub trait HasNext
{
  fn has_next(&self) -> bool;
}

pub trait StreamSpan
{
  type Output;
  fn stream_span(&self) -> Self::Output;
}
