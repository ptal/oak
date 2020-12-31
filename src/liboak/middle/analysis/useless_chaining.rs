// Copyright 2018 Chao Lin & William Sergeant (Sorbonne University)
// Copyright 2020 Pierre Talbot

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This analysis warns about the unnecessary chaining of predicates (!,?) and repeating operators (+,*,?).
//! We suggest to rewrite the expression to equivalent, but simpler version, as follows.
//! There are 4 cases to consider for predicates:
//! !!e -> &e
//! &&e -> &e
//! !&e -> !e
//! &!e -> !e
//!
//! There are 9 cases to consider for repeating operators:
//! e?? -> e?
//! e?+         Taken care of by WFA.
//! e?*         Taken care of by WFA.
//! e+? -> e*
//! e+* -> e+
//! e++ -> e+
//! e*? -> e*
//! e*+         Taken care of by WFA.
//! e**         Taken care of by WFA.
//!
//! The cases where predicates and repeating operators are mixed are taken care of by WFA.

#![macro_use]
use middle::analysis::ast::*;

use self::PredicateOrRepeat::*;

#[derive(PartialEq, Eq, Clone, Copy)]
enum PredicateOrRepeat {
  PAnd,
  PNot,
  POptional,
  POneOrMore,
  PZeroOrMore,
  PNothing
}

impl Default for PredicateOrRepeat {
  fn default() -> Self { PNothing }
}

pub struct UselessChaining<'a>
{
  grammar: &'a AGrammar
}

impl <'a> UselessChaining<'a>
{
  pub fn analyse(grammar: AGrammar) -> Partial<AGrammar> {
    UselessChaining::check_chaining(&grammar);
    Partial::Value(grammar)
  }

  fn check_chaining(grammar: &'a AGrammar){
    let mut analyser = UselessChaining{ grammar };
    for rule in &grammar.rules {
      analyser.visit_expr(rule.expr_idx)
    }
  }

  fn warn_useless_chaining(&self, span: Span, pattern_detected: &'static str, how_to_rewrite: &'static str) -> bool {
    span.unstable().warning(format!(
      "unnecessary chaining of predicates of the form `{}`\n\
       You can rewrite this expression to the equivalent one `{}`.\n\
       ({} ~~~> {})",
      pattern_detected, how_to_rewrite, pattern_detected, how_to_rewrite))
    .emit();
    true
  }

  // Check the chaining pattern given in the documentation above.
  // Additionally, we only suggest to rewrite an inner predicate/repeat if it does not cross a rule.
  // For instance, consider `r = !e`, then calling `!r` might generate a warning, but we do not want to, because it asks the user to modify `r` which might also be used by other rules in other context.
  fn check_chain(&mut self, this: usize, outer: PredicateOrRepeat, inner: PredicateOrRepeat, crossed_rule: bool) -> bool {
    let span = self.grammar[this].span();
    match (outer, inner, crossed_rule) {
      (PNot, PNot, false)=> self.warn_useless_chaining(span, "!(!e)", "&e"),
      (PAnd, PAnd, _) => self.warn_useless_chaining(span, "&(&e)", "&e"),
      (PNot, PAnd, false) => self.warn_useless_chaining(span, "!(&e)", "!e"),
      (PAnd, PNot, _) => self.warn_useless_chaining(span, "&(!e)", "!e"),
      (POptional, POptional, _) => self.warn_useless_chaining(span, "(e?)?", "e?"),
      (POptional, POneOrMore, false) => self.warn_useless_chaining(span, "(e+)?", "e*"),
      (POptional, PZeroOrMore, _) => self.warn_useless_chaining(span, "(e*)?", "e*"),
      (POneOrMore, POneOrMore, _) => self.warn_useless_chaining(span, "(e+)+", "e+"),
      (PZeroOrMore, POneOrMore, _) => self.warn_useless_chaining(span, "(e+)*", "e+"),
      (PZeroOrMore, PZeroOrMore, _) => self.warn_useless_chaining(span, "(e*)*", "e*"),
      (_, _, _) => false // The rest is either valid or taken care of by WFA.
    }
  }

  fn check_chain_and_visit(&mut self, outer: PredicateOrRepeat, this: usize, child: usize) {
    let (inner, crossed_rule) = InnerPredicateOrRepeat::new(self.grammar).visit_expr(child);
    if !self.check_chain(this, outer, inner, crossed_rule) {
      self.visit_expr(child)
    }
  }
}

struct InnerPredicateOrRepeat<'a> {
  grammar: &'a AGrammar
}

impl<'a> InnerPredicateOrRepeat<'a> {
  fn new(grammar: &'a AGrammar) -> Self {
    InnerPredicateOrRepeat { grammar }
  }
}

impl<'a> ExprByIndex for InnerPredicateOrRepeat<'a> {
  fn expr_by_index(&self, index: usize) -> Expression {
    self.grammar.expr_by_index(index).clone()
  }
}

impl<'a> Visitor<(PredicateOrRepeat, bool)> for InnerPredicateOrRepeat<'a> {
  fn visit_non_terminal_symbol(&mut self, _this: usize, rule: &Ident) -> (PredicateOrRepeat, bool) {
    let expr_idx = self.grammar.expr_index_of_rule(rule);
    (self.visit_expr(expr_idx).0, true)
  }

  fn visit_optional(&mut self, _this: usize, _child: usize) -> (PredicateOrRepeat, bool) {
    (POptional, false)
  }

  fn visit_one_or_more(&mut self, _this: usize, _child: usize) -> (PredicateOrRepeat, bool) {
    (POneOrMore, false)
  }

  fn visit_zero_or_more(&mut self, _this: usize, _child: usize) -> (PredicateOrRepeat, bool) {
    (PZeroOrMore, false)
  }

  fn visit_not_predicate(&mut self, _this: usize, _child: usize) -> (PredicateOrRepeat, bool) {
    (PNot, false)
  }

  fn visit_and_predicate(&mut self, _this: usize, _child: usize) -> (PredicateOrRepeat, bool) {
    (PAnd, false)
  }


  fn visit_sequence(&mut self, _: usize, children: Vec<usize>) -> (PredicateOrRepeat, bool) {
    self.visit_expr(children[0])
  }

  fn visit_choice(&mut self, _: usize, children: Vec<usize>) -> (PredicateOrRepeat, bool) {
    let (pred_or_repeat, mut crossed_rule) = self.visit_expr(children[0]);
    for child in children {
      let (pred_or_repeat2, crossed_rule2) = self.visit_expr(child);
      if pred_or_repeat != pred_or_repeat2 {
        return (PNothing, false)
      }
      crossed_rule |= crossed_rule2;
    }
    (pred_or_repeat, crossed_rule)
  }
}

impl<'a> ExprByIndex for UselessChaining<'a>
{
  fn expr_by_index(&self, index: usize) -> Expression {
    self.grammar.expr_by_index(index).clone()
  }
}

impl<'a> Visitor<()> for UselessChaining<'a>
{
  unit_visitor_impl!(choice);
  unit_visitor_impl!(sequence);

  fn visit_one_or_more(&mut self, this: usize, child: usize) {
    self.check_chain_and_visit(POneOrMore, this, child)
  }

  fn visit_zero_or_more(&mut self, this: usize, child: usize){
    self.check_chain_and_visit(PZeroOrMore, this, child)
  }

  fn visit_optional(&mut self, this: usize, child: usize){
    self.check_chain_and_visit(POptional, this, child)
  }

  fn visit_not_predicate(&mut self, this: usize, child: usize){
    self.check_chain_and_visit(PNot, this, child)
  }

  fn visit_and_predicate(&mut self, this: usize, child: usize){
    self.check_chain_and_visit(PAnd, this, child)
  }
}
