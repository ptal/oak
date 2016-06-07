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

pub struct UndeclaredAction<'a>
{
  cx: &'a ExtCtxt<'a>,
  grammar: &'a Grammar,
  has_undeclared: bool
}

impl<'a> UndeclaredAction<'a>
{
  pub fn analyse(cx: &'a ExtCtxt<'a>, grammar: Grammar) -> Partial<Grammar> {
    if UndeclaredAction::has_undeclared(cx, &grammar) {
      Partial::Nothing
    } else {
      Partial::Value(grammar)
    }
  }

  fn has_undeclared(cx: &'a ExtCtxt<'a>, grammar: &Grammar) -> bool {
    let mut analyser = UndeclaredAction {
      cx: cx,
      grammar: grammar,
      has_undeclared: false
    };
    for rule in grammar.rules.values() {
      analyser.visit_expr(rule.def);
    }
    analyser.has_undeclared
  }
}

impl<'a> ExprByIndex for UndeclaredAction<'a>
{
  fn expr_by_index<'b>(&'b self, index: usize) -> &'b Expression {
    self.grammar.expr_by_index(index)
  }
}

impl<'a> Visitor<()> for UndeclaredAction<'a>
{
  unit_visitor_impl!(str_literal);
  unit_visitor_impl!(character);
  unit_visitor_impl!(sequence);
  unit_visitor_impl!(choice);
  unit_visitor_impl!(non_terminal);

  fn visit_semantic_action(&mut self, parent: usize, _expr: usize, id: Ident) {
    if !self.grammar.rust_functions.contains_key(&id) {
      let parent_info = self.grammar.info_by_index(parent);
      self.cx.span_err(parent_info.span, "Undeclared action. This must be a function declared in the grammar scope.");
      self.has_undeclared = true;
    }
  }
}
