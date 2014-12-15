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

use rust::ExtCtxt;
use middle::lint::unused_rule::UnusedRule;
use middle::ast::*;

pub use middle::attribute::ast::Grammar as AGrammar;
pub use middle::attribute::ast::Rule as ARule;
pub use front::ast::Grammar as FGrammar;

mod semantics;
mod lint;
mod attribute;
mod typing;
pub mod ast;

pub fn analyse(cx: &ExtCtxt, fgrammar: FGrammar) -> Option<Grammar>
{
  if !at_least_one_rule_declared(cx, &fgrammar) {
    return None
  }

  // Some(fgrammar)
  //   .and_then(|grammar| FilterItems::analyse(cx, grammar))
  semantics::analyse(cx, fgrammar)
    .and_then(|grammar| AGrammar::new(cx, grammar))
    .and_then(|grammar| UnusedRule::analyse(cx, grammar))
    .and_then(|grammar| typing::grammar_typing(cx, grammar))
}

fn at_least_one_rule_declared(cx: &ExtCtxt, fgrammar: &FGrammar) -> bool
{
  if fgrammar.rules.len() == 0 {
    cx.parse_sess.span_diagnostic.handler.err(
      "At least one rule must be declared.");
    false
  } else {
    true
  }
}
