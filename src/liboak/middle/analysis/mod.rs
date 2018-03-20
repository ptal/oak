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

use front::ast::FGrammar;
use middle::analysis::ast::*;
use middle::analysis::duplicate::*;
use middle::analysis::undeclared_rule::*;
use middle::analysis::undeclared_action::*;
use middle::analysis::well_formedness::*;
use middle::analysis::attribute::*;
use middle::analysis::useless_chaining::*;

mod duplicate;
mod undeclared_rule;
mod undeclared_action;
mod well_formedness;
mod attribute;
mod useless_chaining;
pub mod ast;

pub fn analyse<'a, 'b>(cx: &'a ExtCtxt<'b>, fgrammar: FGrammar) -> Partial<AGrammar<'a, 'b>> {
  let grammar = AGrammar::new(cx, fgrammar.name, fgrammar.exprs, fgrammar.exprs_info);
  let frust_items = fgrammar.rust_items;
  let fattributes = fgrammar.attributes;
  rule_duplicate(grammar, fgrammar.rules)
  .and_then(|grammar| rust_functions_duplicate(grammar, frust_items))
  .and_then(|grammar| UndeclaredRule::analyse(grammar))
  .and_then(|grammar| UndeclaredAction::analyse(grammar))
  .and_then(|grammar| WellFormedness::analyse(grammar))
  .and_then(|grammar| UselessChaining::analyse(grammar))
  .and_then(|grammar| decorate_with_attributes(grammar, fattributes))
}
