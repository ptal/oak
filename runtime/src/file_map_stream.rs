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
use syntex_pos::{BytePos, mk_sp};

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
    let src = filemap.src.as_ref().unwrap();
    FileMapStream::register_lines(filemap, src);
    FileMapStream {
      filemap: filemap.clone(),
      str_stream: (*src).stream()
    }
  }

  fn register_lines(filemap: &Rc<FileMap>, src: &String) {
    // Mostly from Rust compiler (codemap.rs).
    if filemap.count_lines() == 0 {
      let mut byte_pos: u32 = filemap.start_pos.0;
      for line in src.lines() {
        // register the start of this line
        filemap.next_line(BytePos(byte_pos));
        // update byte_pos to include this line and the \n at the end
        byte_pos += line.len() as u32 + 1;
      }
    }
  }

  fn abs_pos(&self) -> BytePos {
    self.filemap.start_pos + BytePos(self.str_stream.bytes_offset() as u32)
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
    mk_sp(
      self.start.abs_pos(),
      self.end.abs_pos()
    )
  }
}

#[cfg(test)]
mod test {
  extern crate syntex_syntax;
  use self::syntex_syntax::codemap::CodeMap;
  use super::*;

  #[test]
  fn test_filemap() {
    let codemap = CodeMap::new();
    let filemap = codemap.new_filemap(format!("fake"), None, format!("A\n\nT\n"));
    let mut stream = filemap.stream();
    assert_eq!(filemap.count_lines(), 3);
    assert!(stream.next() == Some('A'));
    assert!(stream.next() == Some('\n'));
    assert!(stream.next() == Some('\n'));
    assert!(stream.next() == Some('T'));
    // Simulating backtracking
    let mut stream2 = stream.clone();
    assert!(stream.next() == Some('\n'));
    assert!(stream2.next() == Some('\n'));
    assert!(stream.next() == None);
    assert!(stream2.next() == None);
  }
}
