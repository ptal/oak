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

use middle::analysis::ast::*;

pub struct UndeclaredAction<'a: 'c, 'b: 'a, 'c>
{
  grammar: &'c AGrammar<'a, 'b>,
  has_undeclared: bool
}

impl<'a, 'b, 'c> UndeclaredAction<'a, 'b, 'c>
{
  pub fn analyse(grammar: AGrammar<'a, 'b>) -> Partial<AGrammar<'a, 'b>> {
    if UndeclaredAction::has_undeclared(&grammar) {
      Partial::Nothing
    } else {
      Partial::Value(grammar)
    }
  }

  fn has_undeclared(grammar: &'a AGrammar<'a, 'b>) -> bool {
    let mut analyser = UndeclaredAction {
      grammar: grammar,
      has_undeclared: false
    };
    for rule in &grammar.rules {
      analyser.visit_expr(rule.expr_idx);
    }
    analyser.has_undeclared
  }
}

impl<'a, 'b, 'c> ExprByIndex for UndeclaredAction<'a, 'b, 'c>
{
  fn expr_by_index(&self, index: usize) -> Expression {
    self.grammar.expr_by_index(index)
  }
}

impl<'a, 'b, 'c> Visitor<()> for UndeclaredAction<'a, 'b, 'c>
{
  unit_visitor_impl!(str_literal);
  unit_visitor_impl!(atom);
  unit_visitor_impl!(sequence);
  unit_visitor_impl!(choice);
  unit_visitor_impl!(non_terminal);

  fn visit_semantic_action(&mut self, this: usize, _child: usize, action: Ident) {
    if !self.grammar.rust_functions.contains_key(&action) {
      self.grammar.expr_err(
        this,
        format!("Undeclared action `{}`. Function must be declared in the grammar scope.", action)
      );
      self.has_undeclared = true;
    }
  }
}
