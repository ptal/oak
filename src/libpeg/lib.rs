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

#![crate_name = "peg"]
#![experimental]
#![comment = "Parsing Expression Grammar Library"]
#![license = "Apache v2"]
#![crate_type = "dylib"]

#![feature(plugin_registrar, quote, globs)]

extern crate rustc;
extern crate syntax;

use syntax::ext::base::{ExtCtxt, MacResult};
use syntax::codemap::Span;
use syntax::ast::TokenTree;
use rustc::plugin::Registry;

pub use runtime::Parser;

mod utility;
mod ast;
mod semantic_analyser;
mod compiler;

pub mod runtime;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) 
{
  reg.register_macro("peg", expand)
}

fn expand(cx: &mut ExtCtxt, _sp: Span, tts: &[TokenTree]) -> Box<MacResult> 
{
  parse(cx, tts)
}

fn parse(cx: &mut ExtCtxt, tts: &[TokenTree]) -> Box<MacResult>
{
  let mut parser = ast::PegParser::new(cx.parse_sess(), cx.cfg(), Vec::from_slice(tts));
  let peg = parser.parse_grammar();
  let ast = semantic_analyser::SemanticAnalyser::analyse(cx, &peg);
  cx.parse_sess.span_diagnostic.handler.abort_if_errors();
  compiler::PegCompiler::compile(cx, &ast.unwrap())
}

