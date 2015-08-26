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

#[derive(Debug)]
pub struct ParseSuccess<S, T>
{
  pub stream: S,
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
 S: Iterator + Clone
{
  pub fn full_read(&self) -> bool {
    self.stream.clone().peekable().peek().is_some()
  }

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
