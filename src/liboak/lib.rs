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

//! This library proposes a syntax extension for a parser generator based on [Parsing Expression Grammar (PEG)](https://en.wikipedia.org/wiki/Parsing_expression_grammar). It aims at simplifying the construction of the AST by typing the parsing rules. This is an experimental library.

#![feature(convert, rustc_private, plugin_registrar, quote, box_syntax, vec_push_all, drain)]

extern crate rustc;
extern crate syntax;

use rustc::plugin::Registry;

use front::parser;
use monad::partial::Partial;

mod ast;
mod front;
mod middle;
mod back;
mod rust;
mod identifier;
mod monad;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
  reg.register_syntax_extension(
    rust::token::intern("grammar"),
    rust::SyntaxExtension::IdentTT(Box::new(expand), None, false));
}

fn expand<'cx>(cx: &'cx mut rust::ExtCtxt, _sp: rust::Span, grammar_name: rust::Ident,
  tts: Vec<rust::TokenTree>) -> Box<rust::MacResult + 'cx>
{
  parse(cx, grammar_name, tts)
}

fn parse<'cx>(cx: &'cx mut rust::ExtCtxt, grammar_name: rust::Ident,
  tts: Vec<rust::TokenTree>) -> Box<rust::MacResult + 'cx>
{
  let mut parser = parser::Parser::new(cx.parse_sess(), cx.cfg(), tts, grammar_name);
  let ast = parser.parse_grammar();
  let ast = middle::analyse(cx, ast);
  match ast {
    Partial::Value(ast) => back::compile(cx, ast),
    Partial::Fake(_) | Partial::Nothing => {
      cx.parse_sess.span_diagnostic.handler.abort_if_errors();
      rust::DummyResult::any(rust::DUMMY_SP)
    }
  }
}
