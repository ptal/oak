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

//! Compile the typed AST into Rust code.

//! It generates a recognizer and parser function for each rules. It builds the result value and type with the information provided by the AST.

mod ast;
mod naming;
mod function;
mod type_gen;
mod code_gen;
mod code_printer;
mod sum_type;

use middle::typing::ast::Grammar as TGrammar;
use monad::partial::Partial;
use back::code_gen::*;
use back::sum_type::*;
use back::type_gen::*;
use rust;
use rust::ExtCtxt;

pub fn compile<'cx>(cx: &'cx ExtCtxt, tgrammar: TGrammar) -> Partial<Box<rust::MacResult + 'cx>> {
  let grammar = generate_rust_types(cx, tgrammar);
  sum_type_analysis(cx, grammar)
    .and_then(|grammar| generate_rust_code(cx, grammar))
}