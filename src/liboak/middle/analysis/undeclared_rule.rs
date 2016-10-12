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
use partial::Partial::*;

pub struct UndeclaredRule<'a: 'c, 'b: 'a, 'c>
{
  grammar: &'c AGrammar<'a, 'b>,
  has_undeclared: bool
}

impl<'a, 'b, 'c> UndeclaredRule<'a, 'b, 'c>
{
  pub fn analyse(grammar: AGrammar<'a, 'b>) -> Partial<AGrammar<'a, 'b>> {
    if UndeclaredRule::has_undeclared(&grammar) {
      Nothing
    } else {
      Value(grammar)
    }
  }

  fn has_undeclared(grammar: &'c AGrammar<'a, 'b>) -> bool {
    let mut analyser = UndeclaredRule {
      grammar: grammar,
      has_undeclared: false
    };
    for rule in &grammar.rules {
      analyser.visit_expr(rule.expr_idx);
    }
    analyser.has_undeclared
  }
}

impl<'a, 'b, 'c> ExprByIndex for UndeclaredRule<'a, 'b, 'c>
{
  fn expr_by_index(&self, index: usize) -> Expression {
    self.grammar.expr_by_index(index).clone()
  }
}

impl<'a, 'b, 'c> Visitor<()> for UndeclaredRule<'a, 'b, 'c>
{
  unit_visitor_impl!(str_literal);
  unit_visitor_impl!(atom);
  unit_visitor_impl!(sequence);
  unit_visitor_impl!(choice);

  fn visit_non_terminal_symbol(&mut self, this: usize, rule: Ident) {
    let contains_key = self.grammar.rules.iter()
      .find(|r| r.ident() == rule).is_some();
    if !contains_key {
      self.grammar.expr_err(
        this,
        format!("Undeclared rule `{}`.", rule)
      );
      self.has_undeclared = true;
    }
  }
}
