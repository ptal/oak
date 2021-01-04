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

#![feature(proc_macro_diagnostic, proc_macro_span)]

extern crate partial;
extern crate syn;
extern crate quote;
extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod ast;
mod visitor;
mod front;
mod middle;
mod back;
mod identifier;

#[proc_macro]
pub fn oak(input: TokenStream) -> TokenStream {
  let ast = parse_macro_input!(input as front::ast::FGrammar);
  println!("parsing successful!");
  let tast = middle::typecheck(ast);
  println!("typing successful!");
  proc_macro::TokenStream::from(back::compile(tast))
}
