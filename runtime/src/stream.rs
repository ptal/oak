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

pub trait Stream
{
  type Output;
  fn stream(self) -> Self::Output;
}

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

pub trait Location
{
  fn location(&self) -> String;
}

pub trait CodeSnippet
{
  fn code_snippet(&self, len_hint: usize) -> String;
}

pub trait ConsumePrefix<P>
{
  fn consume_prefix(&mut self, prefix: P) -> bool;
}

pub trait HasNext
{
  fn has_next(&self) -> bool;
}
