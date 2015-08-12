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

use middle::analysis::ast::*;
use middle::analysis::duplicate::*;
use middle::analysis::undeclared_rule::*;
use middle::analysis::undeclared_action::*;
use middle::analysis::attribute::*;
use front::ast::Grammar as FGrammar;

mod duplicate;
mod undeclared_rule;
mod undeclared_action;
mod attribute;
pub mod ast;

pub fn analyse(cx: &ExtCtxt, fgrammar: FGrammar) -> Partial<Grammar> {
  Grammar::new(&fgrammar)
    .and_then(|grammar| rule_duplicate(cx, grammar, fgrammar.rules.clone()))
    .and_then(|grammar| rust_item_duplicate(cx, grammar, fgrammar.rust_items.clone()))
    .and_then(|grammar| UndeclaredRule::analyse(cx, grammar))
    .and_then(|grammar| UndeclaredAction::analyse(cx, grammar))
    .and_then(|grammar| decorate_with_attributes(cx, &fgrammar, grammar))
}
