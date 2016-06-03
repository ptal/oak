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

//! This is the developer documentation of Oak, if you do not intend to contribute, please read the [user manual](http://hyc.io/oak) instead. This library proposes a syntax extension for a parser generator based on [Parsing Expression Grammar (PEG)](https://en.wikipedia.org/wiki/Parsing_expression_grammar). It aims at simplifying the construction of the AST by typing the parsing rules. This is an experimental library.

#![feature(rustc_private, plugin_registrar, quote, box_syntax, str_escape)]
#![crate_name = "oak"]

extern crate rustc;
extern crate rustc_plugin;
extern crate syntax;

use rustc_plugin::Registry;

use front::parser;

mod ast;
mod front;
// mod middle;
// mod back;
mod rust;
mod identifier;
mod monad;

type FGrammar = front::ast::Grammar;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
  reg.register_syntax_extension(
    rust::token::intern("grammar"),
    rust::SyntaxExtension::IdentTT(Box::new(expand), None, true));
}

fn expand<'cx>(cx: &'cx mut rust::ExtCtxt, _sp: rust::Span, grammar_name: rust::Ident,
  tts: Vec<rust::TokenTree>) -> Box<rust::MacResult + 'cx>
{
  parse(cx, grammar_name, tts)
}

fn abort_if_errors(cx: &rust::ExtCtxt) {
  cx.parse_sess.span_diagnostic.abort_if_errors();
}

fn unwrap_parser_ast<'a>(cx: &rust::ExtCtxt, ast: rust::PResult<'a, FGrammar>) -> FGrammar {
  match ast {
    Ok(ast) => {
      abort_if_errors(cx);
      ast
    }
    Err(mut err_diagnostic) => {
      err_diagnostic.emit();
      abort_if_errors(cx);
      panic!(rust::FatalError);
    }
  }
}

fn parse<'cx>(cx: &'cx mut rust::ExtCtxt, grammar_name: rust::Ident,
  tts: Vec<rust::TokenTree>) -> Box<rust::MacResult + 'cx>
{
  let parser = parser::Parser::new(cx.parse_sess(), cx.cfg(), tts, grammar_name);
  let ast = parser.parse_grammar();
  let ast = unwrap_parser_ast(cx, ast);
  let cx: &'cx rust::ExtCtxt = cx;
  rust::DummyResult::any(rust::DUMMY_SP)
  // middle::analyse(cx, ast)
  //   .and_next(|ast| back::compile(cx, ast))
  //   .unwrap_or_else(|| {
  //     abort_if_errors(cx);
  //     rust::DummyResult::any(rust::DUMMY_SP)
  //   })
}
