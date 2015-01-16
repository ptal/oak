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

use middle::typing::visitor::*;
use middle::typing::ast::*;

use middle::typing::ast::ExpressionTypeVersion::*;

// The selection phase is used to select the type versions of the future
// parsing functions.
//
// It can be untyped, typed or both depending on the calling contexts.

pub fn selection_phase(cx: &ExtCtxt, grammar: &mut Grammar)
{
  Selector::select(cx, &grammar.rules, grammar.attributes.starting_rule.clone());
}

#[derive(Clone)]
enum TypingContext
{
  CUnTyped,
  CTyped
}

struct Selector<'a>
{
  cx: &'a ExtCtxt,
  rules: &'a HashMap<Ident, Rule>,
  visited: HashMap<Ident, Option<ExpressionTypeVersion>>,
  context: TypingContext
}

impl<'a> Selector<'a>
{
  pub fn select(cx: &'a ExtCtxt, rules: &'a HashMap<Ident, Rule>, start: Ident)
  {
    let mut selector = Selector::new(cx, rules);
    selector.visit_rule(self.rules.get(&start));
  }

  fn new(cx: &'a ExtCtxt, rules: &'a HashMap<Ident, Rule>) -> Selector<'a>
  {
    let mut visited = HashMap::with_capacity(rules.len());
    for (id, rule) in rules.iter() {
      visited.insert(id.clone(), None);
    }
    Selector {
      cx: cx,
      rules: rules,
      visited: visited,
      context: CTyped
    }
  }

  fn is_visited(&self, ident: &Ident) -> bool
  {
    match (*self.visited.get(ident), self.context) {
      (Some(Both), _) => true,
      (Some(Typed), CTyped) => true,
      (Some(UnTyped), CUnTyped) => true,
      _ => false
    }
  }
}

impl<'a> Visitor for Selector<'a>
{
  fn visit_rule(&mut self, rule: &Rule)
  {
    let ident = rule.name.node.clone();
    if !self.is_visited(&ident) {

    }
  }

  fn visit_rule(&mut self, rule: &Rule)
  {
    let ident = rule.name.node.clone();
    *self.visited.get_mut(&ident) = true;
    if rule.is_inline() {
      self.current_inline_path.push(ident);
      walk_rule(self, rule);
      self.current_inline_path.pop();
    } else {
      let current_inline_path = self.current_inline_path.clone();
      self.current_inline_path.clear();
      walk_rule(self, rule);
      self.current_inline_path = current_inline_path;
    }
  }

  // On the (inline) edge.
  fn visit_rule_type_ph(&mut self, _ty: &PTy, ident: Ident)
  {
    if !self.cycle_detected {
      let rule = self.rules.get(&ident);
      let ident = rule.name.node.clone();
      if rule.is_inline() && self.current_inline_path.contains(&ident) {
        self.current_inline_path.push(ident);
        self.loop_detected();
      }
      else if !self.visited.get(&ident) {
        self.visit_rule(rule);
      }
    }
  }

  // Sum type breaks the potential cycles since it cannot be unnamed.
  fn visit_unnamed_sum(&mut self, _parent: &PTy, _inners: &Vec<PTy>)
  {}
}