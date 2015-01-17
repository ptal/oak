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

pub use rust::Span;
pub use middle::semantics::ast::*;

pub trait Visitor
{
  fn visit_grammar(&mut self, grammar: &Grammar)
  {
    walk_grammar(self, grammar);
  }

  fn visit_rule(&mut self, rule: &Rule)
  {
    walk_rule(self, rule);
  }

  fn visit_expr(&mut self, expr: &Box<Expression>)
  {
    walk_expr(self, expr);
  }

  fn visit_str_literal(&mut self, _sp: Span, _lit: &String) {}
  fn visit_any_single_char(&mut self, _sp: Span) {}
  fn visit_non_terminal_symbol(&mut self, _sp: Span, _id: Ident) {}

  fn visit_sequence(&mut self, _sp: Span, exprs: &Vec<Box<Expression>>)
  {
    walk_exprs(self, exprs);
  }

  fn visit_choice(&mut self, _sp: Span, exprs: &Vec<Box<Expression>>)
  {
    walk_exprs(self, exprs);
  }

  fn visit_zero_or_more(&mut self, _sp: Span, expr: &Box<Expression>)
  {
    walk_expr(self, expr);
  }

  fn visit_one_or_more(&mut self, _sp: Span, expr: &Box<Expression>)
  {
    walk_expr(self, expr);
  }

  fn visit_optional(&mut self, _sp: Span, expr: &Box<Expression>)
  {
    walk_expr(self, expr);
  }

  fn visit_not_predicate(&mut self, _sp: Span, expr: &Box<Expression>)
  {
    walk_expr(self, expr);
  }

  fn visit_and_predicate(&mut self, _sp: Span, expr: &Box<Expression>)
  {
    walk_expr(self, expr);
  }

  fn visit_character_class(&mut self, _sp: Span, _expr: &CharacterClassExpr) {}

  fn visit_semantic_action(&mut self, _sp: Span, expr: &Box<Expression>, _id: Ident)
  {
    walk_expr(self, expr);
  }
}

pub fn walk_grammar<V: Visitor+?Sized>(visitor: &mut V, grammar: &Grammar)
{
  for rule in grammar.rules.values() {
    visitor.visit_rule(rule);
  }
}

pub fn walk_rule<V: Visitor+?Sized>(visitor: &mut V, rule: &Rule)
{
  visitor.visit_expr(&rule.def);
}

pub fn walk_expr<V: Visitor+?Sized>(visitor: &mut V, expr: &Box<Expression>)
{
  let sp = expr.span;
  match &expr.node {
    &StrLiteral(ref lit) => {
      visitor.visit_str_literal(sp, lit)
    }
    &AnySingleChar => {
      visitor.visit_any_single_char(sp)
    }
    &NonTerminalSymbol(id) => {
      visitor.visit_non_terminal_symbol(sp, id)
    }
    &Sequence(ref seq) => {
      visitor.visit_sequence(sp, seq)
    }
    &Choice(ref choices) => {
      visitor.visit_choice(sp, choices)
    }
    &ZeroOrMore(ref expr) => {
      visitor.visit_zero_or_more(sp, expr)
    }
    &OneOrMore(ref expr) => {
      visitor.visit_one_or_more(sp, expr)
    }
    &Optional(ref expr) => {
      visitor.visit_optional(sp, expr)
    }
    &NotPredicate(ref expr) => {
      visitor.visit_not_predicate(sp, expr)
    }
    &AndPredicate(ref expr) => {
      visitor.visit_and_predicate(sp, expr)
    }
    &CharacterClass(ref char_class) => {
      visitor.visit_character_class(sp, char_class)
    }
    &SemanticAction(ref expr, id) => {
      visitor.visit_semantic_action(sp, expr, id)
    }
  }
}

pub fn walk_exprs<V: Visitor+?Sized>(visitor: &mut V, exprs: &Vec<Box<Expression>>)
{
  assert!(exprs.len() > 0);
  for expr in exprs.iter() {
    visitor.visit_expr(expr);
  }
}
