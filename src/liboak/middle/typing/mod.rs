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
use middle::typing::bottom_up_unit::*;
// use middle::typing::top_down_unit::*;
// use middle::typing::bottom_up_tuple::*;
// use middle::typing::printer::*;
// use middle::typing::recursive_type::*;

pub mod ast;
mod bottom_up_unit;
// mod bottom_up_tuple;
// mod top_down_unit;
// mod recursive_type;
// mod printer;

pub fn type_inference<'cx>(agrammar: AGrammar<'cx>) -> Partial<TGrammar<'cx>> {
  let grammar = TGrammar::typed_grammar(agrammar);
  let grammar = UnitTyping::infer(grammar);
  Partial::Value(grammar)
  // top_down_unit_inference(&mut grammar);
  // print_annotated_rules(&grammar);
  // recursive_type_analysis(cx, grammar)
  //   .and_then(|grammar| bottom_up_tuple_inference(grammar))
}
