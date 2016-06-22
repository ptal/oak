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

//! The recursive type analysis ensures that mutual recursive rules that need to be typed can actually be typed.

use middle::typing::ast::*;
use monad::partial::Partial;

pub fn recursive_type_analysis<'a>(cx: &'a ExtCtxt<'a>, grammar: Grammar)
  -> Partial<Grammar>
{
  if RecursiveType::analyse(cx, &grammar.rules) {
    Partial::Value(grammar)
  }
  else {
    Partial::Nothing
  }
}

pub struct RecursiveType<'a>
{
  cx: &'a ExtCtxt<'a>,
  rules: &'a HashMap<Ident, Rule>,
  visited: HashMap<Ident, bool>,
  current_inline_path: Vec<Ident>,
  cycle_detected: bool,
  /// This boolean stays true if we only forward type along the recursive cycle. In this case, it means that no new value must be built.
  forwarding_type: bool
}

impl<'a> RecursiveType<'a>
{
  fn analyse(cx: &'a ExtCtxt<'a>, rules: &'a HashMap<Ident, Rule>) -> bool {
    let mut inlining_loop = RecursiveType::new(cx, rules);
    inlining_loop.visit_rules();
    !inlining_loop.cycle_detected
  }

  fn new(cx: &'a ExtCtxt<'a>, rules: &'a HashMap<Ident, Rule>) -> RecursiveType<'a> {
    let mut visited = HashMap::with_capacity(rules.len());
    for id in rules.keys() {
      visited.insert(id.clone(), false);
    }
    RecursiveType {
      cx: cx,
      rules: rules,
      visited: visited,
      current_inline_path: vec![],
      cycle_detected: false,
      forwarding_type: true
    }
  }

  fn visit_rules(&mut self) {
    for rule in self.rules.values() {
      if self.cycle_detected {
        break;
      }
      self.current_inline_path.clear();
      self.visit_rule(rule);
    }
  }

  fn visit_rule(&mut self, rule: &Rule) {
    let ident = rule.name.node;
    *self.visited.get_mut(&ident).unwrap() = true;
    if !rule.def.is_unit() {
      self.current_inline_path.push(ident);
      self.visit_expr(&rule.def);
      self.current_inline_path.pop();
    }
  }

  fn loop_detected(&mut self) {
    self.cycle_detected = true;
    let in_cycle = self.current_inline_path.pop().unwrap();
    // Consider the smallest cycle which is garantee since we extract the element that closed the cycle.
    let mut trimmed_cycle = vec![in_cycle];
    for id in self.current_inline_path.iter().rev() {
      if *id == in_cycle {
        break;
      }
      trimmed_cycle.push(id.clone());
    }

    let mut db = self.cx.struct_span_err(self.rules.get(&in_cycle).unwrap().name.span,
      "Inlining cycle detected. \
      The type of a rule must be inlined into itself (indirectly or not), which is impossible.");
    for cycle_node in trimmed_cycle.iter() {
      db.span_note(self.rules.get(cycle_node).unwrap().name.span,
        "This rule is part of the recursive type.");
    }
    db.note("Recursive data types are not handled automatically, \
      you must create it yourself with a semantic action.\nIf you don't care about the value of this rule, \
      annotate it with `rule = (e) -> ()` or annotate leaf rules that produce values with `rule = (e) -> (^)`.");
    db.emit();
  }
}

impl<'a> Visitor<Expression, ()> for RecursiveType<'a>
{
  unit_visitor_impl!(Expression, str_literal);
  unit_visitor_impl!(Expression, character);
  unit_visitor_impl!(Expression, sequence);
  unit_visitor_impl!(Expression, choice);

  fn visit_non_terminal_symbol(&mut self, _parent: &Box<Expression>, ident: Ident) {
    if !self.cycle_detected {
      let rule = self.rules.get(&ident).unwrap();
      let ident = rule.name.node;
      if !rule.def.is_unit() && self.current_inline_path.contains(&ident) && !self.forwarding_type {
        self.current_inline_path.push(ident);
        self.loop_detected();
      }
      else if !*self.visited.get(&ident).unwrap() {
        self.visit_rule(rule);
      }
    }
  }

  /// Base case: Expression with a unit type does not generate a recursive type.
  /// If the current expression is not only a projection, it means that a type must be built.
  fn visit_expr(&mut self, expr: &Box<Expression>) {
    if !expr.is_unit() {
      if !expr.is_forwading_type() {
        let forwading_old = self.forwarding_type;
        self.forwarding_type = false;
        walk_expr(self, expr);
        self.forwarding_type = forwading_old;
      }
      else {
        walk_expr(self, expr);
      }
    }
  }

  /// Base case: Semantic actions always have type given by the user, so recursivity is handled by the user.
  fn visit_semantic_action(&mut self, _parent: &Box<Expression>,
    _expr: &Box<Expression>, _id: Ident)
  {}
}
