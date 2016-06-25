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

pub fn type_inference<'a, 'b>(agrammar: AGrammar<'a, 'b>) -> Partial<TGrammar<'a, 'b>> {
  println!("naive typing...");
  let grammar = TGrammar::typed_grammar(agrammar);
  println!("unit inference...");
  let grammar = UnitInference::infer(grammar);
  println!("recursive type analysis...");
  RecursiveType::analyse(grammar).map(|grammar|
    TupleUnpacking::infer(grammar)
  )
}
