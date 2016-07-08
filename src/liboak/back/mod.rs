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

mod context;
mod continuation;
mod name_factory;
mod code_printer;
mod compiler;

use middle::typing::ast::*;
use rust;

pub fn compile<'a, 'b>(grammar: TGrammar<'a, 'b>)
  -> Partial<Box<rust::MacResult + 'a>>
{
  Partial::Value(compiler::GrammarCompiler::compile(grammar))
}
