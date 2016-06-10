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

pub use syntax::ptr::P;
pub use syntax::ast;
pub use syntax::ast::*;
pub use syntax::print::pprust::*;
pub use syntax::print::pp;
pub use syntax::util::small_vector::SmallVector;
pub use syntax::codemap::{DUMMY_SP, Span, MultiSpan, Spanned, spanned, mk_sp, respan, BytePos};
pub use syntax::errors::*;
pub use syntax::ext::base::{ExtCtxt,MacResult,MacEager,DummyResult};
pub use syntax::ext::quote::rt::ToTokens;
pub use syntax::ext::build::AstBuilder;
pub use syntax::ext::base::SyntaxExtension;

pub use syntax::parse::str_lit;
pub use syntax::parse::parser::Parser;
pub use syntax::parse::ParseSess;
pub use syntax::parse::PResult;
pub use syntax::parse::new_parser_from_tts;
pub use syntax::parse::token::str_to_ident;
pub use syntax::parse::token::Token;
pub use syntax::parse::token;
pub use syntax::parse::token::keywords::Keyword;
pub use syntax::parse::token::DelimToken;
pub use syntax::parse::token::BinOpToken;
pub use syntax::parse::token::gensym_ident;
