// Copyright 2015 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Bottom-up inference of tuple types. It will unpack every possible tuple type, therefore type of the form `(T1, (T2, T3))` are flatten into `(T1, T2, T3)`. This transformation always terminates since we checked for recursive type in the `typing::recursive_type` analysis.

use middle::typing::ast::*;

pub struct TupleUnpacking<'a, 'b: 'a>
{
  grammar: TGrammar<'a, 'b>,
  reached_fixpoint: bool
}

impl<'a, 'b> TupleUnpacking<'a, 'b>
{
  pub fn infer(grammar: TGrammar<'a, 'b>) -> TGrammar<'a, 'b> {
    let mut engine = TupleUnpacking::new(grammar);
    let rules = engine.grammar.rules
      .values()
      .map(|rule| rule.expr_idx)
      .collect();
    engine.compute_fixpoint(rules);
    engine.grammar
  }

  fn new<'cx>(grammar: TGrammar<'a, 'b>) -> TupleUnpacking<'a, 'b> {
    TupleUnpacking {
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

impl<'a, 'b> ExprByIndex for TupleUnpacking<'a, 'b>
{
  fn expr_by_index(&self, index: usize) -> Expression {
    self.grammar.expr_by_index(index)
  }
}

impl<'a, 'b> Visitor<()> for TupleUnpacking<'a, 'b>
{
  unit_visitor_impl!(str_literal);
  unit_visitor_impl!(character);
  unit_visitor_impl!(any_single_char);
  unit_visitor_impl!(character_class);
  unit_visitor_impl!(non_terminal);
  unit_visitor_impl!(sequence);
  unit_visitor_impl!(choice);

  fn visit_expr(&mut self, expr: usize) {
    match self.grammar[expr].tuple_indexes() {
      Some(indexes) => {
        let unpacked_indexes = self.unpack_tuple(indexes);
        self.grammar[expr].to_tuple_type(unpacked_indexes);
      }
      _ => ()
    }
    walk_expr(self, expr);
  }
}

impl<'a, 'b> TupleUnpacking<'a, 'b>
{
  fn unpack_tuple(&mut self, indexes: Vec<usize>) -> Vec<usize> {
    let mut unpacked_indexes = vec![];
    for idx in indexes {
      match self.grammar[idx].tuple_indexes() {
        Some(indexes) => {
          unpacked_indexes.extend(indexes.into_iter());
          self.reached_fixpoint = false;
        }
        None => {
          unpacked_indexes.push(idx);
        }
      }
    }
    unpacked_indexes
  }
}
