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

pub struct RecursiveType<'cx>
{
  grammar: TGrammar<'cx>,
  visited: HashMap<Ident, bool>,
  current_inline_path: Vec<Ident>,
  cycle_detected: bool,
  /// This boolean becomes true if we must automatically generate a value.
  value_built: bool
}

impl<'cx> RecursiveType<'cx>
{
  pub fn analyse(grammar: TGrammar) -> Partial<TGrammar> {
    let mut engine = RecursiveType::new(grammar);
    engine.visit_rules();
    if engine.cycle_detected {
      Partial::Nothing
    }
    else {
      Partial::Value(engine.grammar)
    }
  }

  fn new(grammar: TGrammar<'cx>) -> RecursiveType<'cx> {
    let mut visited = HashMap::with_capacity(grammar.rules.len());
    for id in grammar.rules.keys() {
      visited.insert(id.clone(), false);
    }
    RecursiveType {
      grammar: grammar,
      visited: visited,
      current_inline_path: vec![],
      cycle_detected: false,
      value_built: false
    }
  }

  fn visit_rules(&mut self) {
    let rules: Vec<_> = self.grammar.rules.values().cloned().collect();
    for rule in rules {
      if self.cycle_detected {
        break;
      }
      self.current_inline_path.clear();
      self.visit_rule(rule);
    }
  }

  fn visit_rule(&mut self, rule: Rule) {
    let ident = rule.ident();
    *self.visited.get_mut(&ident).unwrap() = true;
    if !self.grammar[rule.expr_idx].ty.is_unit() {
      self.current_inline_path.push(ident);
      self.visit_expr(rule.expr_idx);
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

    let mut errors = vec![(
      self.grammar.rules[&in_cycle].span(),
      format!("Inlining cycle detected. \
      The type of a rule must be inlined into itself (indirectly or not), which is impossible.")
    )];
    for cycle_node in trimmed_cycle.iter() {
      errors.push((
        self.grammar.rules[cycle_node].span(),
        format!("This rule is part of the recursive type.")));
    }
    errors.push((
      self.grammar.rules[&in_cycle].span(),
      format!("Recursive data types are not handled automatically, \
      you must create it yourself with a semantic action.\nIf you don't care about the value of this rule, \
      annotate it with `rule = e -> ()` or annotate leaf rules that produce values with `rule = e -> (^)`.")));
    self.grammar.multi_locations_err(errors);
  }
}

impl<'a> ExprByIndex for RecursiveType<'a>
{
  fn expr_by_index(&self, index: usize) -> Expression {
    self.grammar.expr_by_index(index).clone()
  }
}

impl<'a> Visitor<()> for RecursiveType<'a>
{
  unit_visitor_impl!(str_literal);
  unit_visitor_impl!(character);
  unit_visitor_impl!(sequence);
  unit_visitor_impl!(choice);

  /// Base case on type:
  /// `Tuple(t1,t2,...)` or `Identity`: We must generate a type, so there is a risk of recursivity.
  /// `Tuple(t1)`: Projection of the sub-expression type, it does not break any recursivity cycle, therefore we must explore more.
  /// `Tuple()` or host-type: It breaks recursivity cycle, so we do not explore more.
  fn visit_expr(&mut self, expr_idx: usize) {
    if self.grammar[expr_idx].ty.is_value_constructor() {
      let value_built_old = self.value_built;
      self.value_built = true;
      walk_expr(self, expr_idx);
      self.value_built = value_built_old;
    }
    else if self.grammar[expr_idx].ty.is_projection() {
      walk_expr(self, expr_idx);
    }
  }

  fn visit_non_terminal_symbol(&mut self, _parent: usize, ident: Ident) {
    if !self.cycle_detected {
      let rule = self.grammar.rules[&ident].clone();
      let ident = rule.ident();
      if self.current_inline_path.contains(&ident) && self.value_built {
        self.current_inline_path.push(ident);
        self.loop_detected();
      }
      else if !self.visited[&ident] {
        self.visit_rule(rule);
      }
    }
  }
}
