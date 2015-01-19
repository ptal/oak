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
use middle::typing::inlining::*;
use middle::typing::propagation::*;
use middle::typing::ast::*;
use monad::partial::Partial;

pub mod ast;
pub mod inference;
mod visitor;
mod inlining;
mod inlining_loop;
mod propagation;

pub fn grammar_typing(cx: &ExtCtxt, agrammar: AGrammar) -> Partial<Grammar>
{
  let mut grammar = Grammar {
    name: agrammar.name,
    rules: HashMap::with_capacity(agrammar.rules.len()),
    rust_items: agrammar.rust_items,
    attributes: agrammar.attributes
  };
  InferenceEngine::infer(cx, &mut grammar, agrammar.rules);
  inlining_phase(cx, &mut grammar);
  propagation_phase(&mut grammar);
  Partial::Value(grammar)
}
