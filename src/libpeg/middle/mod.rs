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

use rust::{ExtCtxt, Span};
use middle::visitor::Visitor;
use middle::lint::unused_rule::UnusedRule;

use middle::ast::*;

mod lint;
mod visitor;
mod attribute;
pub mod ast;

pub fn analyse(cx: &ExtCtxt, fgrammar: FGrammar) -> Option<Grammar>
{
  if !at_least_one_rule_declared(cx, &fgrammar) {
    return None
  }

  Grammar::new(cx, fgrammar)
    .and_then(|grammar| UndeclaredRule::analyse(cx, grammar))
    .and_then(|grammar| UnusedRule::analyse(cx, grammar))
}

fn at_least_one_rule_declared(cx: &ExtCtxt, fgrammar: &FGrammar) -> bool
{
  if fgrammar.rules.len() == 0 {
    cx.parse_sess.span_diagnostic.handler.err(
      "At least one rule must be declared.");
    false
  } else {
    true
  }
}

struct UndeclaredRule<'a>
{
  cx: &'a ExtCtxt<'a>,
  rules: &'a HashMap<Ident, Rule>,
  has_undeclared: bool
}

impl<'a> UndeclaredRule<'a>
{
  fn analyse(cx: &'a ExtCtxt<'a>, grammar: Grammar) -> Option<Grammar>
  {
    if UndeclaredRule::has_undeclared(cx, &grammar) {
      None
    } else {
      Some(grammar)
    }
  }

  fn has_undeclared(cx: &'a ExtCtxt<'a>, grammar: &Grammar) -> bool
  {
    let mut analyser = UndeclaredRule {
      cx: cx,
      rules: &grammar.rules,
      has_undeclared: false
    };
    analyser.visit_grammar(grammar);
    analyser.has_undeclared
  }
}

impl<'a> Visitor for UndeclaredRule<'a>
{
  fn visit_non_terminal_symbol(&mut self, sp: Span, id: Ident)
  {
    if !self.rules.contains_key(&id) {
      self.cx.span_err(sp, "Undeclared rule.");
      self.has_undeclared = true;
    }
  }
}
