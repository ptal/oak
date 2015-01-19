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

pub use middle::typing::ast::*;

use middle::typing::ast::ExpressionType::*;
use rust;

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

  fn visit_character(&mut self) {}
  fn visit_unit(&mut self) {}
  fn visit_unit_propagate(&mut self, _parent: &PTy) {}
  fn visit_rule_type_ph(&mut self, _parent: &PTy, _ident: Ident) {}
  fn visit_vector(&mut self, _parent: &PTy, _inner: &PTy) {}
  fn visit_tuple(&mut self, _parent: &PTy, _inners: &Vec<PTy>) {}
  fn visit_optional_ty(&mut self, _parent: &PTy, _inner: &PTy) {}
  fn visit_unnamed_sum(&mut self, _parent: &PTy, _inners: &Vec<PTy>) {}
  fn visit_action_ty(&mut self, _parent: &PTy, _inner: &rust::FunctionRetTy) {}
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
  walk_expr_node(visitor, &expr.node, expr.span.clone());
  walk_ty(visitor, &expr.ty);
}

pub fn walk_expr_node<V: Visitor+?Sized>(visitor: &mut V, expr: &ExpressionNode, sp: Span)
{
  match expr {
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

pub fn walk_ty<V: Visitor+?Sized>(visitor: &mut V, ty: &PTy)
{
  // We don't want to borrow for the entire exploration, it'd
  // prevent mutable borrow.
  let ty_rc = {
    let ty_ref = ty.borrow();
    ty_ref.clone()
  };
  match &*ty_rc {
    &Character => visitor.visit_character(),
    &Unit => visitor.visit_unit(),
    &UnitPropagate => visitor.visit_unit_propagate(ty),
    &RuleTypePlaceholder(ref id) => visitor.visit_rule_type_ph(ty, id.clone()),
    &Vector(ref sub_ty) => visitor.visit_vector(ty, sub_ty),
    &Tuple(ref sub_tys) => visitor.visit_tuple(ty, sub_tys),
    &OptionalTy(ref sub_ty) => visitor.visit_optional_ty(ty, sub_ty),
    &UnnamedSum(ref sub_tys) => visitor.visit_unnamed_sum(ty, sub_tys),
    &Action(ref rust_ty) => visitor.visit_action_ty(ty, rust_ty)
  }
}
