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

use middle::typing::inference::*;
use middle::typing::bottom_up_unit::*;
use middle::typing::top_down_unit::*;
use middle::typing::ast::*;
use monad::partial::Partial;

pub mod ast;
pub mod visitor;
mod inference;
mod bottom_up_unit;
mod top_down_unit;
// mod analysis;

pub fn type_inference(agrammar: AGrammar) -> Partial<Grammar>
{
  let mut grammar = Grammar {
    name: agrammar.name,
    rules: HashMap::with_capacity(agrammar.rules.len()),
    rust_items: agrammar.rust_items,
    attributes: agrammar.attributes
  };
  InferenceEngine::infer(&mut grammar, agrammar.rules);
  bottom_up_unit_inference(&mut grammar);
  top_down_unit_inference(&mut grammar);
  Partial::Value(grammar)
}


pub fn type_analysis(_cx: &ExtCtxt, grammar: Grammar) -> Partial<Grammar>
{
  Partial::Value(grammar)
}
