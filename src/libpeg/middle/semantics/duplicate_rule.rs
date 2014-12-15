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

pub use front::ast::{Expression_, Expression, CharacterInterval, CharacterClassExpr};
pub use front::ast::Expression_::*;

pub use rust::{ExtCtxt, Span, Spanned, SpannedIdent};
pub use identifier::*;
pub use std::collections::HashMap;

use front::ast::Rule as FRule;
use middle::semantics::ast::*;

pub struct DuplicateRule<'a>
{
  cx: &'a ExtCtxt<'a>,
  grammar: Grammar,
  has_duplicate: bool
}

impl<'a> DuplicateRule<'a>
{
  pub fn analyse(cx: &'a ExtCtxt<'a>, grammar: Grammar, frules: Vec<FRule>) -> Option<Grammar>
  {
    DuplicateRule {
      cx: cx,
      grammar: grammar,
      has_duplicate: false
    }.populate(frules)
     .make()
  }

  fn populate(mut self, frules: Vec<FRule>) -> DuplicateRule<'a>
  {
    for rule in frules.into_iter() {
      let name = rule.name.node.clone();
      if self.grammar.rules.contains_key(&name) {
        self.duplicate_rules(self.grammar.rules.get(&name).unwrap(), &rule);
        self.has_duplicate = true;
      } else {
        self.grammar.rules.insert(name, rule);
      }
    }
    self
  }

  fn duplicate_rules(&self, pre: &FRule, current: &FRule)
  {
    self.cx.span_err(current.name.span, "Duplicate rule definition.");
    self.cx.span_note(pre.name.span, "Previous declaration here.");
  }

  fn make(self) -> Option<Grammar>
  {
    if self.has_duplicate {
      None
    } else {
      Some(self.grammar)
    }
  }
}