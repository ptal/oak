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

use rust::ExtCtxt;
use middle::semantics::visitor::*;

pub struct UndeclaredAction<'a>
{
  cx: &'a ExtCtxt<'a>,
  grammar: &'a Grammar,
  has_undeclared: bool
}

impl<'a> UndeclaredAction<'a>
{
  pub fn analyse(cx: &'a ExtCtxt<'a>, grammar: Grammar) -> Partial<Grammar>
  {
    if UndeclaredAction::has_undeclared(cx, &grammar) {
      Partial::Nothing
    } else {
      Partial::Value(grammar)
    }
  }

  fn has_undeclared(cx: &'a ExtCtxt<'a>, grammar: &Grammar) -> bool
  {
    let mut analyser = UndeclaredAction {
      cx: cx,
      grammar: grammar,
      has_undeclared: false
    };
    analyser.visit_grammar(grammar);
    analyser.has_undeclared
  }
}

impl<'a> Visitor for UndeclaredAction<'a>
{
  fn visit_semantic_action(&mut self, sp: Span, _expr: &Box<Expression>, id: Ident)
  {
    if !self.grammar.rust_items.contains_key(&id) {
      self.cx.span_err(sp, "Undeclared action. This must be a function declared in the grammar scope.");
      self.has_undeclared = true;
    }
  }
}
