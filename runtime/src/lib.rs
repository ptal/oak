// Copyright 2014 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This is the documentation of the Oak runtime. Oak is a parser generator of _Parsing Expression Grammar_, please read first the [manual](http://hyc.io/oak).
//!
//! This library is used by the generated code of Oak and is also necessary to any Oak users for interfacing with the code generated.
//! A PEG combinator returns a `ParseState`, please consult the methods `into_result` or `unwrap_data` as they are good starting point for retrieving useful information.

extern crate syntex_pos;

pub use str_stream::*;
pub use stream::*;
pub use parse_state::*;
use syntex_pos::{BytePos, mk_sp};

pub mod str_stream;
pub mod parse_state;
pub mod stream;
pub mod file_map_stream;

pub fn make_span(lo: usize, hi: usize) -> Span {
  mk_sp(
    BytePos(lo as u32),
    BytePos(hi as u32))
}
