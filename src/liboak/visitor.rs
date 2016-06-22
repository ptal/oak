// Copyright 2016 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![macro_use]

use ast::*;
use ast::Expression::*;

pub trait Visitor<R> : ExprByIndex
{
  fn visit_expr(&mut self, expr: usize) -> R {
    walk_expr(self, expr)
  }

  fn visit_str_literal(&mut self, _parent: usize, _lit: String) -> R;
  fn visit_non_terminal_symbol(&mut self, _parent: usize, _id: Ident) -> R;
  fn visit_character(&mut self, _parent: usize) -> R;

  fn visit_any_single_char(&mut self, parent: usize) -> R {
    self.visit_character(parent)
  }

  fn visit_character_class(&mut self, parent: usize, _expr: CharacterClassExpr) -> R {
    self.visit_character(parent)
  }

  fn visit_sequence(&mut self, _parent: usize, exprs: Vec<usize>) -> R;
  fn visit_choice(&mut self, _parent: usize, exprs: Vec<usize>) -> R;

  fn visit_repeat(&mut self, _parent: usize, expr: usize) -> R {
    walk_expr(self, expr)
  }

  fn visit_zero_or_more(&mut self, parent: usize, expr: usize) -> R {
    self.visit_repeat(parent, expr)
  }

  fn visit_one_or_more(&mut self, parent: usize, expr: usize) -> R {
    self.visit_repeat(parent, expr)
  }

  fn visit_optional(&mut self, _parent: usize, expr: usize) -> R {
    walk_expr(self, expr)
  }

  fn visit_syntactic_predicate(&mut self, _parent: usize, expr: usize) -> R {
    walk_expr(self, expr)
  }

  fn visit_not_predicate(&mut self, parent: usize, expr: usize) -> R {
    self.visit_syntactic_predicate(parent, expr)
  }

  fn visit_and_predicate(&mut self, parent: usize, expr: usize) -> R {
    self.visit_syntactic_predicate(parent, expr)
  }

  fn visit_semantic_action(&mut self, _parent: usize, expr: usize, _id: Ident) -> R {
    walk_expr(self, expr)
  }
}

/// We need this macro for factorizing the code since we can not specialize a trait on specific type parameter (we would need to specialize on `()` here).
macro_rules! unit_visitor_impl {
  (str_literal) => (fn visit_str_literal(&mut self, _parent: usize, _lit: String) -> () {});
  (non_terminal) => (fn visit_non_terminal_symbol(&mut self, _parent: usize, _id: Ident) -> () {});
  (character) => (fn visit_character(&mut self, _parent: usize) -> () {});
  (any_single_char) => (fn visit_any_single_char(&mut self, _parent: usize) -> () {});
  (character_class) => (fn visit_character_class(&mut self, _parent: usize, _expr: CharacterClassExpr) -> () {});
  (sequence) => (
    fn visit_sequence(&mut self, _parent: usize, exprs: Vec<usize>) -> () {
      walk_exprs(self, exprs);
    }
  );
  (choice) => (
    fn visit_choice(&mut self, _parent: usize, exprs: Vec<usize>) -> () {
      walk_exprs(self, exprs);
    }
  );
}

pub fn walk_expr<R, V: ?Sized>(visitor: &mut V, parent: usize) -> R where
  V: Visitor<R>
{
  match visitor.expr_by_index(parent) {
    StrLiteral(lit) => {
      visitor.visit_str_literal(parent, lit)
    }
    AnySingleChar => {
      visitor.visit_any_single_char(parent)
    }
    NonTerminalSymbol(id) => {
      visitor.visit_non_terminal_symbol(parent, id)
    }
    Sequence(seq) => {
      visitor.visit_sequence(parent, seq)
    }
    Choice(choices) => {
      visitor.visit_choice(parent, choices)
    }
    ZeroOrMore(expr) => {
      visitor.visit_zero_or_more(parent, expr)
    }
    OneOrMore(expr) => {
      visitor.visit_one_or_more(parent, expr)
    }
    Optional(expr) => {
      visitor.visit_optional(parent, expr)
    }
    NotPredicate(expr) => {
      visitor.visit_not_predicate(parent, expr)
    }
    AndPredicate(expr) => {
      visitor.visit_and_predicate(parent, expr)
    }
    CharacterClass(char_class) => {
      visitor.visit_character_class(parent, char_class)
    }
    SemanticAction(expr, id) => {
      visitor.visit_semantic_action(parent, expr, id)
    }
  }
}

pub fn walk_exprs<R, V: ?Sized>(visitor: &mut V, exprs: Vec<usize>) -> Vec<R> where
  V: Visitor<R>
{
  exprs.into_iter().map(|expr| visitor.visit_expr(expr)).collect()
}
