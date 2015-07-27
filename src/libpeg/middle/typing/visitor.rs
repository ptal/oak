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

use middle::typing::ast::*;

pub trait Visitor<R>
{
  fn visit_expr(&mut self, expr: &Box<Expression>) -> R {
    walk_expr(self, expr)
  }

  fn visit_str_literal(&mut self, _parent: &Box<Expression>, _lit: &String) -> R;
  fn visit_non_terminal_symbol(&mut self, _parent: &Box<Expression>, _id: Ident) -> R;

  fn visit_character(&mut self, _parent: &Box<Expression>) -> R;

  fn visit_any_single_char(&mut self, parent: &Box<Expression>) -> R {
    self.visit_character(parent)
  }

  fn visit_character_class(&mut self, parent: &Box<Expression>, _expr: &CharacterClassExpr) -> R {
    self.visit_character(parent)
  }

  fn visit_sequence(&mut self, _parent: &Box<Expression>, exprs: &Vec<Box<Expression>>) -> R;
  fn visit_choice(&mut self, _parent: &Box<Expression>, exprs: &Vec<Box<Expression>>) -> R;

  fn visit_repeat(&mut self, _parent: &Box<Expression>, expr: &Box<Expression>) -> R {
    walk_expr(self, expr)
  }

  fn visit_zero_or_more(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) -> R {
    self.visit_repeat(parent, expr)
  }

  fn visit_one_or_more(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) -> R {
    self.visit_repeat(parent, expr)
  }

  fn visit_optional(&mut self, _parent: &Box<Expression>, expr: &Box<Expression>) -> R {
    walk_expr(self, expr)
  }

  fn visit_syntactic_predicate(&mut self, _parent: &Box<Expression>, expr: &Box<Expression>) -> R {
    walk_expr(self, expr)
  }

  fn visit_not_predicate(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) -> R {
    self.visit_syntactic_predicate(parent, expr)
  }

  fn visit_and_predicate(&mut self, parent: &Box<Expression>, expr: &Box<Expression>) -> R {
    self.visit_syntactic_predicate(parent, expr)
  }

  fn visit_semantic_action(&mut self, _parent: &Box<Expression>, expr: &Box<Expression>, _id: Ident) -> R {
    walk_expr(self, expr)
  }
}

pub fn walk_expr<R, V: Visitor<R>+?Sized>(visitor: &mut V, parent: &Box<Expression>) -> R
{
  match &parent.node {
    &StrLiteral(ref lit) => {
      visitor.visit_str_literal(parent, lit)
    }
    &AnySingleChar => {
      visitor.visit_any_single_char(parent)
    }
    &NonTerminalSymbol(id) => {
      visitor.visit_non_terminal_symbol(parent, id)
    }
    &Sequence(ref seq) => {
      visitor.visit_sequence(parent, seq)
    }
    &Choice(ref choices) => {
      visitor.visit_choice(parent, choices)
    }
    &ZeroOrMore(ref expr) => {
      visitor.visit_zero_or_more(parent, expr)
    }
    &OneOrMore(ref expr) => {
      visitor.visit_one_or_more(parent, expr)
    }
    &Optional(ref expr) => {
      visitor.visit_optional(parent, expr)
    }
    &NotPredicate(ref expr) => {
      visitor.visit_not_predicate(parent, expr)
    }
    &AndPredicate(ref expr) => {
      visitor.visit_and_predicate(parent, expr)
    }
    &CharacterClass(ref char_class) => {
      visitor.visit_character_class(parent, char_class)
    }
    &SemanticAction(ref expr, id) => {
      visitor.visit_semantic_action(parent, expr, id)
    }
  }
}

pub fn walk_exprs<R, V: Visitor<R>+?Sized>(visitor: &mut V, exprs: &Vec<Box<Expression>>) -> Vec<R>
{
  exprs.iter().map(|expr| visitor.visit_expr(expr)).collect()
}
