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
    FileMapStream {
      filemap: filemap.clone(),
      str_stream: (*filemap.src.as_ref().unwrap()).stream()
    }
  }

  fn abs_pos(&self) -> BytePos {
    BytePos(self.str_stream.bytes_offset() as u32) + self.filemap.start_pos
  }
}

impl<'a> Iterator for FileMapStream<'a>
{
  type Item = char;
  /// Mostly from Rust compiler (libsyntax/parse/lexer/mod.rs::bump()).
  fn next(&mut self) -> Option<Self::Item> {
    let old_pos = self.abs_pos();
    let old_ch = self.str_stream.current_char();
    self.str_stream.next().map(|c| {
      let pos = self.abs_pos();
      if old_ch.unwrap() == '\n' {
        self.filemap.next_line(pos);
      }
      let byte_offset_diff = (pos.0 - old_pos.0) as usize;
      if byte_offset_diff > 1 {
        self.filemap.record_multibyte_char(old_pos, byte_offset_diff);
      }
      c
    })
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
    let filemap = codemap.new_filemap(format!("fake"), None, format!("\nT\n"));
    let mut stream = filemap.stream();
    assert!(stream.next() == Some('\n'));
    assert!(stream.next() == Some('T'));
    assert!(stream.next() == Some('\n'));
    assert!(stream.next() == None);
  }
}
