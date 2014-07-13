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

use ast::*;
use utility::*;

use syntax::ext::base::ExtCtxt;
use syntax::codemap::Span;

pub fn check_peg(cx: &ExtCtxt, peg: &Peg)
{
  let mut starting_rule = None;
  for rule in peg.rules.iter() {
    check_rule_rhs(cx, peg, &rule.def);
    if check_start_attribute(cx, &starting_rule, rule) {
      starting_rule = Some(rule);
    }
  }
  match starting_rule {
    None =>
      cx.parse_sess.span_diagnostic.handler.warn(
        "No rule has been specified as the starting point (attribute `#[start]`). The first rule will be automatically considered as such."),
    _ => ()
  }
}

fn span_err(cx: &ExtCtxt, sp: Span, m: &str) 
{
  cx.parse_sess.span_diagnostic.span_err(sp, m);
}

fn check_start_attribute<'a>(cx: &ExtCtxt, starting_rule: &Option<&'a Rule>, rule: &'a Rule) -> bool
{
  let start_attr = start_attribute(&rule.attributes);
  match start_attr {
    Some(ref attr) => {
      match starting_rule {
        &None => true,
        &Some(starting_rule) => {
          span_err(cx, attr.span, format!(
            "Multiple `start` attributes are forbidden. Rules `{}` and `{}` conflict.",
            id_to_string(starting_rule.name),
            id_to_string(rule.name)).as_slice());
          false
        }
      }
    },
    _ => false
  }
}

fn check_rule_rhs(cx: &ExtCtxt, peg: &Peg, expr: &Box<Expression>)
{
  match &expr.node {
    &NonTerminalSymbol(id) => {
      check_non_terminal_symbol(cx, peg, id, expr.span)
    }
    &Sequence(ref seq) => {
      check_expr_slice(cx, peg, seq.as_slice())
    }
    &Choice(ref choices) => {
      check_expr_slice(cx, peg, choices.as_slice())
    }
    _ => ()
  }
}

fn check_non_terminal_symbol(cx: &ExtCtxt, peg: &Peg, id: Ident, sp: Span)
{
  check_if_rule_is_declared(cx, peg, id, sp)
}

fn check_if_rule_is_declared(cx: &ExtCtxt, peg: &Peg, id: Ident, sp: Span)
{
  for rule in peg.rules.iter() {
    if rule.name == id {
      return;
    }
  }
  span_err(cx, sp, 
    format!("You try to call the rule `{}` which is not declared.", id_to_string(id)).as_slice());
}

fn check_expr_slice<'a>(cx: &ExtCtxt, peg: &Peg, seq: &'a [Box<Expression>])
{
  assert!(seq.len() > 0);
  for expr in seq.iter() {
    check_rule_rhs(cx, peg, expr);
  }
}
