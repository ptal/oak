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

use middle::attribute::visitor::*;
pub use rust::ExtCtxt;

pub struct UnusedRule<'a>
{
  rules: &'a HashMap<Ident, Rule>,
  is_used: HashMap<Ident, bool>
}

impl<'a> UnusedRule<'a>
{
  pub fn analyse(cx: &ExtCtxt, grammar: Grammar) -> Option<Grammar>
  {
    let is_used = UnusedRule::launch(&grammar);
    UnusedRule::remove_unused(cx, grammar, is_used)
  }

  fn launch(grammar: &Grammar) -> HashMap<Ident, bool>
  {
    let mut analyser = UnusedRule::new(&grammar.rules);
    analyser.launch_dfs(&grammar.attributes.starting_rule);
    analyser.is_used
  }

  fn new(rules: &'a HashMap<Ident, Rule>) -> UnusedRule<'a>
  {
    let mut analyser = UnusedRule {
      rules: rules,
      is_used: HashMap::new()
    };
    for k in rules.keys() {
      analyser.is_used.insert(k.clone(), false);
    }
    analyser
  }

  fn launch_dfs(&mut self, start: &Ident)
  {
    *self.is_used.get_mut(start).unwrap() = true;
    self.visit_rule(self.rules.get(start).unwrap());
  }

  fn remove_unused(cx: &ExtCtxt, grammar: Grammar,
    is_used: HashMap<Ident, bool>) -> Option<Grammar>
  {
    let mut grammar = grammar;
    for (id, &used) in is_used.iter() {
      if !used {
        let rule = grammar.rules.remove(id).unwrap();
        cx.span_warn(rule.name.span, "Unused rule.");
      }
    }
    Some(grammar)
  }

  fn mark_rule(&mut self, id: Ident) -> bool
  {
    let used = self.is_used.get_mut(&id).unwrap();
    if !*used {
      *used = true;
      true
    } else {
      false
    }
  }
}

impl<'a> Visitor for UnusedRule<'a>
{
  fn visit_non_terminal_symbol(&mut self, _sp: Span, id: Ident)
  {
    if self.mark_rule(id) {
      self.visit_rule(self.rules.get(&id).unwrap());
    }
  }
}
