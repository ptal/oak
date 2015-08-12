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
use std::collections::HashMap;

pub struct UndeclaredRule<'a>
{
  cx: &'a ExtCtxt<'a>,
  rules: &'a HashMap<Ident, Rule>,
  has_undeclared: bool
}

impl<'a> UndeclaredRule<'a>
{
  pub fn analyse(cx: &'a ExtCtxt<'a>, grammar: Grammar) -> Partial<Grammar> {
    if UndeclaredRule::has_undeclared(cx, &grammar) {
      Nothing
    } else {
      Value(grammar)
    }
  }

  fn has_undeclared(cx: &'a ExtCtxt<'a>, grammar: &Grammar) -> bool {
    let mut analyser = UndeclaredRule {
      cx: cx,
      rules: &grammar.rules,
      has_undeclared: false
    };
    for rule in grammar.rules.values() {
      analyser.visit_expr(&rule.def);
    }
    analyser.has_undeclared
  }
}

impl<'a> Visitor<Expression, ()> for UndeclaredRule<'a>
{
  unit_visitor_impl!(Expression, str_literal);
  unit_visitor_impl!(Expression, character);
  unit_visitor_impl!(Expression, sequence);
  unit_visitor_impl!(Expression, choice);

  fn visit_non_terminal_symbol(&mut self, parent: &Box<Expression>, id: Ident) {
    if !self.rules.contains_key(&id) {
      self.cx.span_err(parent.span, "Undeclared rule.");
      self.has_undeclared = true;
    }
  }
}
