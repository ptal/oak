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

mod ast;
mod naming;
mod function;
mod type_gen;
mod code_gen;
mod code_printer;

use middle::ast::Grammar as TGrammar;
use back;
use rust;
use rust::ExtCtxt;

pub fn compile<'cx>(cx: &'cx ExtCtxt, tgrammar: TGrammar) -> Box<rust::MacResult + 'cx> {
  let grammar = back::type_gen::generate_rust_types(cx, tgrammar);
  back::code_gen::generate_rust_code(cx, grammar)
}