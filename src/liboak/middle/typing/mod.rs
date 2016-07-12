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

use middle::analysis::ast::AGrammar;
use middle::typing::ast::*;
use middle::typing::unit_inference::*;
use middle::typing::recursive_type::*;
use middle::typing::tuple_unpacking::*;

pub mod ast;
mod unit_inference;
mod recursive_type;
mod tuple_unpacking;
mod typing_printer;

pub fn type_inference<'a, 'b>(agrammar: AGrammar<'a, 'b>) -> Partial<TGrammar<'a, 'b>> {
  let grammar = TGrammar::typed_grammar(agrammar);
  grammar.print_debug_typing("TGrammar::typed_grammar");
  let grammar = UnitInference::infer(grammar);
  grammar.print_debug_typing("UnitInference::infer");
  RecursiveType::analyse(grammar).map(|grammar| {
    let grammar = TupleUnpacking::infer(grammar);
    grammar.print_debug_typing("TupleUnpacking::infer");
    grammar.print_typing();
    grammar
  })
}
