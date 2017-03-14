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

use stream::*;
use str_stream::*;
use std::rc::*;
use std::cmp::Ordering;
pub use std::ops::Range;
pub use syntex_pos::{Span, FileMap};
// pub use compiler_toolbox::FileMap;

impl<'a> Stream for &'a Rc<FileMap>
{
  type Output = FileMapStream<'a>;
  fn stream(self) -> Self::Output {
    FileMapStream::new(self)
  }
}

#[derive(Clone)]
pub struct FileMapStream<'a>
{
  filemap: Rc<FileMap>,
  str_stream: StrStream<'a>,
}

impl<'a> FileMapStream<'a>
{
  fn new(filemap: &'a Rc<FileMap>) -> Self {
    FileMapStream {
      filemap: filemap.clone(),
      str_stream: (*filemap.src.as_ref().unwrap()).stream()
    }
  }
}

impl<'a> Iterator for FileMapStream<'a>
{
  type Item = char;
  fn next(&mut self) -> Option<Self::Item> {
    self.str_stream.next()
  }
}

impl<'a> PartialEq for FileMapStream<'a>
{
  fn eq(&self, other: &Self) -> bool {
    (&self.str_stream).eq(&other.str_stream)
  }
}

impl<'a> Eq for FileMapStream<'a> {}

impl<'a> PartialOrd for FileMapStream<'a>
{
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    (&self.str_stream).partial_cmp(&other.str_stream)
  }
}

impl<'a> Ord for FileMapStream<'a>
{
  fn cmp(&self, other: &Self) -> Ordering {
    (&self.str_stream).cmp(&other.str_stream)
  }
}

impl<'a> Location for FileMapStream<'a>
{
  fn location(&self) -> String {
    self.str_stream.location()
  }
}

impl<'a> CodeSnippet for FileMapStream<'a>
{
  fn code_snippet(&self, len_hint: usize) -> String {
    self.str_stream.code_snippet(len_hint)
  }
}

impl<'a> ConsumePrefix<&'static str> for FileMapStream<'a>
{
  fn consume_prefix(&mut self, prefix: &'static str) -> bool {
    self.str_stream.consume_prefix(prefix)
  }
}

impl<'a> HasNext for FileMapStream<'a>
{
  fn has_next(&self) -> bool {
    self.str_stream.has_next()
  }
}

impl<'a> StreamSpan for Range<FileMapStream<'a>>
{
  type Output = Span;
  fn stream_span(&self) -> Self::Output {
    let mut span = Range {
      start: self.start.str_stream.clone(),
      end: self.end.str_stream.clone()
    }.stream_span();
    span.lo = span.lo + self.start.filemap.start_pos;
    span.hi = span.hi + self.end.filemap.end_pos;
    span
  }
}
