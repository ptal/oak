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

pub use Producer;
use std::str::CharIndices;
use std::cmp::Ordering;

impl<'a> Producer for &'a str
{
  type Stream = StrStream<'a>;
  fn producer(self) -> StrStream<'a> {
    StrStream::new(self)
  }
}

#[derive(Clone)]
pub struct StrStream<'a>
{
  raw_data: &'a str,
  current: CharIndices<'a>
}

impl<'a> StrStream<'a>
{
  fn new(raw_data: &'a str) -> StrStream<'a> {
    StrStream {
      raw_data: raw_data,
      current: raw_data.char_indices()
    }
  }

  fn offset(&self) -> usize {
    match self.current.clone().peekable().next() {
      None => self.raw_data.len(),
      Some((idx, _)) => idx
    }
  }

  #[inline(always)]
  fn assert_same_raw_data(&self, other: &StrStream<'a>) {
    debug_assert!(self.raw_data.as_ptr() == other.raw_data.as_ptr(),
      "Operations between two streams are only defined when they share the same raw data.");
  }
}

impl<'a> Iterator for StrStream<'a>
{
  type Item = char;
  fn next(&mut self) -> Option<Self::Item> {
    self.current.next().map(|(_,x)| x)
  }
}

impl<'a> PartialEq for StrStream<'a>
{
  fn eq(&self, other: &Self) -> bool {
    self.assert_same_raw_data(other);
    self.offset() == other.offset()
  }
}

impl<'a> Eq for StrStream<'a> {}

impl<'a> PartialOrd for StrStream<'a>
{
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    self.assert_same_raw_data(other);
    self.offset().partial_cmp(&other.offset())
  }
}

impl<'a> Ord for StrStream<'a>
{
  fn cmp(&self, other: &Self) -> Ordering {
    self.assert_same_raw_data(other);
    self.offset().cmp(&other.offset())
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_stream() {
    let abc = "abc";
    let mut s1 = abc.producer();
    let s1_init = s1.clone();
    let mut s2 = s1_init.clone();
    for c in abc.chars() {
      assert!(s1 == s2);
      assert_eq!(s1.next().unwrap(), c);
      assert!(s1 > s1_init);
      assert!(s1 > s2);
      s2 = s1.clone();
    }
    assert_eq!(s1.next(), None);
    assert_eq!(s2.next(), None);
    assert!(s1 > s1_init);
    assert!(s1 == s2);
  }

  #[test]
  fn test_empty_stream() {
    let mut empty = "".producer();
    assert_eq!(empty.offset(), 0);
    assert_eq!(empty.next(), None);
    assert_eq!(empty.next(), None);
    assert_eq!(empty.offset(), 0);
    assert!(empty == empty);
    assert!(!(empty > empty));
    let empty2 = empty.clone();
    assert!(empty == empty2);
    assert!(!(empty > empty2));
  }

  fn test_unrelated_streams<R, F>(op: F) where
   F: FnOnce(&StrStream<'static>, &StrStream<'static>) -> R
  {
    let s1 = "abc".producer();
    let s2 = "def".producer();
    op(&s1, &s2);
  }

  #[test]
  #[should_panic]
  fn unrelated_stream_eq() {
    test_unrelated_streams(|a, b| a == b);
  }

  #[test]
  #[should_panic]
  fn unrelated_stream_partial_ord() {
    test_unrelated_streams(|a, b| a.partial_cmp(b));
  }

  #[test]
  #[should_panic]
  fn unrelated_stream_ord() {
    test_unrelated_streams(|a, b| a.cmp(b));
  }
}
