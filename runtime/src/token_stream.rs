// Copyright 2018 Marin Dupas (UPMC)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Implementation of `Stream` for `TokenStream` type. It implements all traits required by `CharStream`.
//!
extern crate proc_macro;
use self::proc_macro::TokenTree;
use self::proc_macro::TokenStream;

use stream::*;
use make_span;
use std::cmp::{Ordering};
pub use std::ops::Range;
pub use syntex_pos::Span;

impl Stream for TokenStream
{
    type Output = OTokenStream;
    fn stream<>(self) -> OTokenStream {
        OTokenStream::new(self)
    }
}

/// Represents a stream from a `TokenStream`. It implements all traits required by `CharStream`.
#[derive(Clone)]
pub struct OTokenStream
{
    tokens: Vec<TokenTree>,
    offset: usize
}

impl OTokenStream
{
    fn new(tokens: TokenStream) -> OTokenStream {
        OTokenStream {
            tokens: tokens.into_iter().collect(),
            offset: 0
        }
    }


    #[inline(always)]
    fn assert_same_raw_data(&self, other: &OTokenStream) {
        debug_assert!(self.tokens.as_ptr() == other.tokens.as_ptr(),
                      "Operations between two streams are only defined when they share the same raw data.");
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn current_token(&self) -> &TokenTree {
        &self.tokens[self.offset]
    }

    pub fn line_column(&self) -> (usize, usize) {
        (self.current_token().span.start().line,
         self.current_token().span.start().column)
    }
}

impl Iterator for OTokenStream
{
    type Item = TokenTree;
    fn next(&mut self) -> Option<Self::Item> {
        if self.has_next() {
            let current = self.current_token().clone();
            self.offset += 1;
            Some(current)
        } else {
            None
        }
    }
}

impl PartialEq for OTokenStream
{
    fn eq(&self, other: &Self) -> bool {
        self.assert_same_raw_data(other);
        self.offset == other.offset
    }
}

impl Eq for OTokenStream {}

impl PartialOrd for OTokenStream
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.assert_same_raw_data(other);
        self.offset.partial_cmp(&other.offset)
    }
}

impl Ord for OTokenStream
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.assert_same_raw_data(other);
        self.offset.cmp(&other.offset)
    }
}


impl Location for OTokenStream
{
    fn location(&self) -> String {
        let (line, column) = self.line_column();
        format!("{}:{}", line, column)
    }
}

impl CodeSnippet for OTokenStream
{
    fn code_snippet(&self, _len_hint: usize) -> String {
        if ! self.has_next() {
            String::from("<end-of-file>")
        }
        else {
            self.current_token().to_string()
        }
    }
}

impl ConsumePrefix<Vec<TokenTree>> for OTokenStream
{
    fn consume_prefix(&mut self, prefix: Vec<TokenTree>) -> bool { //TODO : consume like prefix
        self.offset += 1;
        self.has_next()
    }
}

impl HasNext for OTokenStream
{
    fn has_next(&self) -> bool {
        self.offset < self.tokens.len()
    }
}

impl StreamSpan for Range<OTokenStream> //TODO : verify if it can be better
{
    type Output = Span;
    fn stream_span(&self) -> Self::Output {
        make_span(
            self.start.offset,
            self.end.offset)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn consume_prefix_test(stream: &OTokenStream, prefix: Vec<TokenTree>,
                               prefix_match: bool, next_token: Option<TokenTree>)
    {
        let mut s2 = stream.clone();
        assert_eq!(s2.consume_prefix(prefix), prefix_match);
        assert!(s2.next() == next_token);
    }

    #[test]
    fn test_consume_prefix() {
        let s1 = OTokenStream {
            tokens: [TokenTree{
                span: make_span(0,1),
                kind: TokenNode {Literal: 'a'}
            },
                TokenTree{
                    span: make_span(0,2),
                    kind: TokenNode {Literal: 'b'}
                },
                TokenTree{
                    span: make_span(0,3),
                    kind: TokenNode {Literal: 'c'}
                }],
            offset: 0
        };
        consume_prefix_test(s1, [TokenTree{
            span: make_span(0,1),
            kind: TokenNode {Literal: 'a'}
        },
            TokenTree{
                span: make_span(0,2),
                kind: TokenNode {Literal: 'b'}
            },
            TokenTree{
                span: make_span(0,3),
                kind: TokenNode {Literal: 'c'}
            }], true, None);
        consume_prefix_test(s1, [TokenTree{
            span: make_span(0,1),
            kind: TokenNode {Literal: 'a'}
        },
            TokenTree{
                span: make_span(0,2),
                kind: TokenNode {Literal: 'b'}
            }], true, Some(TokenTree{
            span: make_span(0,1),
            kind: TokenNode {Literal: 'c'}
        }));
        consume_prefix_test(s1, [], true, Some(TokenTree{
            span: make_span(0,1),
            kind: TokenNode {Literal: 'a'}
        }));
        consume_prefix_test(s1, [TokenTree{
            span: make_span(0,1),
            kind: TokenNode {Literal: 'a'}
        },
            TokenTree{
                span: make_span(0,3),
                kind: TokenNode {Literal: 'c'}
            }], false, Some(TokenTree{
            span: make_span(0,1),
            kind: TokenNode {Literal: 'a'}
        }));
        consume_prefix_test(s1, [TokenTree{
            span: make_span(0,1),
            kind: TokenNode {Literal: 'z'}
        }], false, Some(TokenTree{
            span: make_span(0,1),
            kind: TokenNode {Literal: 'a'}
        }));
    }
/*
    fn test_str_stream<'a, I>(mut s1: StrStream<'a>, chars: I) where
        I: Iterator<Item=char>
    {
        let s1_init = s1.clone();
        let mut s2 = s1_init.clone();
        for c in chars {
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
    fn test_stream() {
        let abc = "abc";
        test_str_stream(abc.stream(), abc.chars());
    }

    #[test]
    fn test_string_stream() {
        let abc = String::from("abc");
        test_str_stream(abc.stream(), abc.chars());
    }

    #[test]
    fn test_empty_stream() {
        let mut empty = "".stream();
        assert_eq!(empty.bytes_offset, 0);
        assert_eq!(empty.next(), None);
        assert_eq!(empty.next(), None);
        assert_eq!(empty.bytes_offset, 0);
        assert!(empty == empty);
        assert!(!(empty > empty));
        let empty2 = empty.clone();
        assert!(empty == empty2);
        assert!(!(empty > empty2));
    }

    fn test_unrelated_streams<R, F>(op: F) where
        F: FnOnce(&StrStream<'static>, &StrStream<'static>) -> R
    {
        let s1 = "abc".stream();
        let s2 = "def".stream();
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
    }*/
}