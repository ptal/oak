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
use middle::analysis::resolve_non_terminal::*;
use middle::analysis::well_formedness::*;
use middle::analysis::attribute::*;
use middle::analysis::useless_chaining::*;
// use middle::analysis::unreachable_rule::*;

mod duplicate;
mod resolve_non_terminal;
mod well_formedness;
mod attribute;
mod useless_chaining;
// mod unreachable_rule;
pub mod ast;

pub fn analyse(fgrammar: FGrammar) -> Partial<AGrammar> {
  let grammar = AGrammar::new(fgrammar.start_span, fgrammar.exprs, fgrammar.exprs_info);
  let frust_items = fgrammar.rust_items;
  let fattributes = fgrammar.attributes;
  rule_duplicate(grammar, fgrammar.rules)
  .and_then(|grammar| rust_functions_duplicate(grammar, frust_items))
  .and_then(|grammar| ResolveNonTerminal::resolve(grammar))
  .and_then(|grammar| WellFormedness::analyse(grammar))
  .and_then(|grammar| UselessChaining::analyse(grammar))
  // .and_then(|grammar| UnreachableRule::analyse(grammar))   // This analysis must be reviewed and fixed.
  .and_then(|grammar| decorate_with_attributes(grammar, fattributes))
}
