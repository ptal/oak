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
use monad::partial::Partial::*;

pub struct UndeclaredRule<'a>
{
  grammar: &'a AGrammar<'a>,
  has_undeclared: bool
}

impl<'a> UndeclaredRule<'a>
{
  pub fn analyse(grammar: AGrammar) -> Partial<AGrammar> {
    if UndeclaredRule::has_undeclared(&grammar) {
      Nothing
    } else {
      Value(grammar)
    }
  }

  fn has_undeclared(grammar: &AGrammar) -> bool {
    let mut analyser = UndeclaredRule {
      grammar: grammar,
      has_undeclared: false
    };
    for rule in grammar.rules.values() {
      analyser.visit_expr(rule.expr_idx);
    }
    analyser.has_undeclared
  }
}

impl<'a> ExprByIndex for UndeclaredRule<'a>
{
  fn expr_by_index(&self, index: usize) -> Expression {
    self.grammar.expr_by_index(index).clone()
  }
}

impl<'a> Visitor<()> for UndeclaredRule<'a>
{
  unit_visitor_impl!(str_literal);
  unit_visitor_impl!(character);
  unit_visitor_impl!(sequence);
  unit_visitor_impl!(choice);

  fn visit_non_terminal_symbol(&mut self, parent: usize, id: Ident) {
    if !self.grammar.rules.contains_key(&id) {
      self.grammar.expr_err(
        parent,
        format!("Undeclared rule `{}`.", id)
      );
      self.has_undeclared = true;
    }
  }
}
