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

//! Unit inference consists of propagating unit types and invisibility.
//!
//! The typing rules are of the form `expr:ty => expr':ty'` which means that if `expr` has type `ty` then `expr'` has type `ty'`:
//! * Basic combinators (`e*`, `e+`, `e?`):
//!    * `f(e:(^)) => f(e):(^)`
//!    * `f(e:t) => f(e):Identity`
//! * Syntactic predicates (`&e`, `!e`):
//!    * `f(e:t) => f(e):(^)`
//! * Semantics actions: `(e:t > g) => (e > g): Action`.
//! * Non terminal symbol (`R` being a function from rule identifier to type)
//!    * `ident:Identity => ident:(^)` if `R(ident) = (^)`.
//!    * `ident:Identity => ident:()` if `R(ident) = ()`.
//! * Sequence (symmetric cases not shown, easily generalizable for n-tuples):
//!    * `e:t e':t' => (e e'): (t, t')`
//!    * `(e e'): ((^), (^)) => (e e'): (^)`
//!    * `(e e'): (t, (^)) => (e e'): t`
//!    * `(e e'): (t, ()) => (e e'): t`
//! * Choice:
//!    * `e:t / e':t => (e / e'): t` if `t` is equal to `()` or `(^)`
//!    * `e:t / e':t' => (e / e'):Ìdentity` if `t=t'`
//!    * `e:(^) / e':() => (e / e'): ()`
//! * Explicit typing operator `->`:
//!    * `e:t -> () => e:()`
//!    * `e:t -> (^) => e:(^)`
//!

use middle::typing::ast::*;

pub struct UnitInference<'a, 'b: 'a>
{
  grammar: TGrammar<'a, 'b>,
  reached_fixpoint: bool
}

impl<'a, 'b> UnitInference<'a, 'b>
{
  pub fn infer(grammar: TGrammar<'a, 'b>) -> TGrammar<'a, 'b> {
    let mut engine = UnitInference::new(grammar);
    let rules = engine.grammar.rules
      .values()
      .map(|rule| rule.expr_idx)
      .collect();
    engine.compute_fixpoint(rules);
    engine.grammar
  }

  fn new(grammar: TGrammar<'a, 'b>) -> UnitInference<'a, 'b> {
    UnitInference {
      grammar: grammar,
      reached_fixpoint: false
    }
  }

  fn compute_fixpoint(&mut self, rules: Vec<usize>) {
    while !self.reached_fixpoint {
      self.reached_fixpoint = true;
      for expr_idx in &rules {
        self.visit_expr(*expr_idx);
      }
    }
  }
}

impl<'a, 'b> ExprByIndex for UnitInference<'a, 'b>
{
  fn expr_by_index(&self, index: usize) -> Expression {
    self.grammar.expr_by_index(index)
  }
}

impl<'a, 'b> Visitor<()> for UnitInference<'a, 'b>
{
  unit_visitor_impl!(str_literal);
  unit_visitor_impl!(character);
  unit_visitor_impl!(any_single_char);
  unit_visitor_impl!(character_class);

  fn visit_repeat(&mut self, parent: usize, child: usize) {
    self.propagate(parent, child);
  }

  fn visit_optional(&mut self, parent: usize, child: usize) {
    self.propagate(parent, child);
  }

  fn visit_non_terminal_symbol(&mut self, parent: usize, id: Ident) {
    let rule_expr = self.grammar.expr_index_of_rule(id);
    self.propagate_invisibility(rule_expr, parent);
  }

  fn visit_sequence(&mut self, parent: usize, children: Vec<usize>) {
    self.propagate_all(parent, children.clone());
    let indexes = self.grammar[parent].tuple_indexes()
      .expect("A sequence can only be typed as a tuple.");
    if self.all_invisible(&indexes) {
      self.invisible(parent);
    }
    else {
      let indexes = indexes.into_iter()
        .filter(|&idx| !self.grammar[idx].ty.is_unit())
        .collect();
      self.tuple(parent, indexes);
    }
  }

  fn visit_choice(&mut self, parent: usize, children: Vec<usize>) {
    self.propagate_all(parent, children.clone());
    if self.all_invisible(&children) {
      self.invisible(parent);
    }
    else if self.all_unit(&children) {
      self.unit(parent);
    }
  }
}

impl<'a, 'b> UnitInference<'a, 'b>
{
  fn invisible(&mut self, expr_idx: usize) {
    if !self.grammar[expr_idx].is_invisible() {
      self.grammar[expr_idx].to_invisible_type();
      self.reached_fixpoint = false;
    }
  }

  fn unit(&mut self, expr_idx: usize) {
    if !self.grammar[expr_idx].ty.is_unit() {
      self.grammar[expr_idx].to_unit_type();
      self.reached_fixpoint = false;
    }
  }

  fn tuple(&mut self, expr_idx: usize, indexes: Vec<usize>) {
    if !self.grammar[expr_idx].eq_tuple_indexes(&indexes) {
      self.grammar[expr_idx].to_tuple_type(indexes);
      self.reached_fixpoint = false;
    }
  }

  fn propagate_unit(&mut self, source: usize, target: usize) {
    if self.grammar[source].ty.is_unit() {
      self.unit(target);
    }
  }

  fn propagate_invisibility(&mut self, source: usize, target: usize) {
    if self.grammar[source].is_invisible() {
      self.invisible(target);
    }
    else {
      self.propagate_unit(source, target);
    }
  }

  fn propagate(&mut self, parent: usize, child: usize) {
    self.propagate_unit(parent, child);
    walk_expr(self, child);
    self.propagate_invisibility(child, parent);
  }

  fn propagate_all(&mut self, parent: usize, children: Vec<usize>) {
    for child in &children {
      self.propagate_unit(parent, *child);
    }
    walk_exprs(self, children);
  }

  fn all_invisible(&self, exprs_indexes: &Vec<usize>) -> bool {
    if exprs_indexes.len() == 0 {
      false
    }
    else {
      exprs_indexes.iter().all(|&idx| self.grammar[idx].is_invisible())
    }
  }

  fn all_unit(&self, exprs_indexes: &Vec<usize>) -> bool {
    exprs_indexes.iter().all(|&idx| self.grammar[idx].ty.is_unit())
  }
}
