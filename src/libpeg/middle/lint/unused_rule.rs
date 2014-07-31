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

use middle::visitor::*;
use rust::{ExtCtxt, Ident};
use std::collections::hashmap::HashMap;

pub struct UnusedRule<'a>
{
  cx: &'a ExtCtxt<'a>,
  ident_to_rule_idx: &'a HashMap<Ident, uint>,
  grammar: &'a Grammar,
  pub is_used: Vec<bool>
}

impl<'a> UnusedRule<'a>
{
  pub fn new(cx: &'a ExtCtxt<'a>, grammar: &'a Grammar,
    ident_to_rule_idx: &'a HashMap<Ident, uint>) -> UnusedRule<'a>
  {
    UnusedRule {
      cx: cx,
      ident_to_rule_idx: ident_to_rule_idx,
      grammar: grammar,
      is_used: Vec::from_fn(grammar.rules.len(), |_| false)
    }
  }

  pub fn analyse(&mut self, start_rule_idx: uint)
  {
    *self.is_used.get_mut(start_rule_idx) = true;
    self.visit_rule(&self.grammar.rules[start_rule_idx]);
    for (idx, used) in self.is_used.iter().enumerate() {
      if !used {
        let sp = self.grammar.rules[idx].name.span;
        self.cx.parse_sess.span_diagnostic.span_warn(sp, 
          format!("The rule `{}` is not used.",
            id_to_string(self.grammar.rules[idx].name.node)).as_slice());
      }
    }
  }
}

impl<'a> Visitor for UnusedRule<'a>
{
  fn visit_non_terminal_symbol(&mut self, _sp: Span, id: Ident)
  {
    let idx = *self.ident_to_rule_idx.find(&id).unwrap();
    if !self.is_used[idx] {
      *self.is_used.get_mut(idx) = true;
      self.visit_rule(&self.grammar.rules[idx]);
    }
  }
}
