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

use middle::semantics::ast::*;
use middle::semantics::duplicate_rule::*;
use middle::semantics::duplicate_rust_item::*;
use middle::semantics::undeclared_rule::*;
use front::ast::Grammar as FGrammar;

mod duplicate;
mod duplicate_rule;
mod duplicate_rust_item;
mod undeclared_rule;
pub mod ast;
pub mod visitor;

pub fn analyse(cx: &ExtCtxt, fgrammar: FGrammar) -> Partial<Grammar>
{
  Grammar::new(&fgrammar)
    .and_then(|grammar| rule_duplicate(cx, grammar, fgrammar.rules.clone()))
    .and_then(|grammar| rust_item_duplicate(cx, grammar, fgrammar.rust_items.clone()))
    .and_then(|grammar| UndeclaredRule::analyse(cx, grammar))
}
