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
  fn visit_expr(&mut self, this: usize) -> R {
    walk_expr(self, this)
  }

  fn visit_str_literal(&mut self, _this: usize, _lit: String) -> R;
  fn visit_non_terminal_symbol(&mut self, _this: usize, _rule: Ident) -> R;
  fn visit_atom(&mut self, _this: usize) -> R;

  fn visit_any_single_char(&mut self, this: usize) -> R {
    self.visit_atom(this)
  }

  fn visit_character_class(&mut self, this: usize, _char_class: CharacterClassExpr) -> R {
    self.visit_atom(this)
  }

  fn visit_spanned_expr(&mut self, _this: usize, child: usize) -> R {
    self.visit_expr(child)
  }

  fn visit_sequence(&mut self, _this: usize, children: Vec<usize>) -> R;
  fn visit_choice(&mut self, _this: usize, children: Vec<usize>) -> R;

  fn visit_repeat(&mut self, _this: usize, child: usize) -> R {
    self.visit_expr(child)
  }

  fn visit_zero_or_more(&mut self, this: usize, child: usize) -> R {
    self.visit_repeat(this, child)
  }

  fn visit_one_or_more(&mut self, this: usize, child: usize) -> R {
    self.visit_repeat(this, child)
  }

  fn visit_optional(&mut self, _this: usize, child: usize) -> R {
    self.visit_expr(child)
  }

  fn visit_syntactic_predicate(&mut self, _this: usize, child: usize) -> R {
    self.visit_expr(child)
  }

  fn visit_not_predicate(&mut self, this: usize, child: usize) -> R {
    self.visit_syntactic_predicate(this, child)
  }

  fn visit_and_predicate(&mut self, this: usize, child: usize) -> R {
    self.visit_syntactic_predicate(this, child)
  }

  fn visit_semantic_action(&mut self, _this: usize, child: usize, _action: Ident) -> R {
    self.visit_expr(child)
  }

  fn visit_type_ascription(&mut self, _this: usize, child: usize, _ty: IType) -> R {
    self.visit_expr(child)
  }
}

/// We need this macro for factorizing the code since we can not specialize a trait on specific type parameter (we would need to specialize on `()` here).
macro_rules! unit_visitor_impl {
  (str_literal) => (fn visit_str_literal(&mut self, _this: usize, _lit: String) -> () {});
  (non_terminal) => (fn visit_non_terminal_symbol(&mut self, _this: usize, _rule: Ident) -> () {});
  (atom) => (fn visit_atom(&mut self, _this: usize) -> () {});
  (any_single_char) => (fn visit_any_single_char(&mut self, _this: usize) -> () {});
  (character_class) => (fn visit_character_class(&mut self, _this: usize, _char_class: CharacterClassExpr) -> () {});
  (sequence) => (
    fn visit_sequence(&mut self, _this: usize, children: Vec<usize>) -> () {
      walk_exprs(self, children);
    }
  );
  (choice) => (
    fn visit_choice(&mut self, _this: usize, children: Vec<usize>) -> () {
      walk_exprs(self, children);
    }
  );
}

pub fn walk_expr<R, V: ?Sized>(visitor: &mut V, this: usize) -> R where
  V: Visitor<R>
{
  match visitor.expr_by_index(this) {
    StrLiteral(lit) => {
      visitor.visit_str_literal(this, lit)
    }
    AnySingleChar => {
      visitor.visit_any_single_char(this)
    }
    NonTerminalSymbol(rule) => {
      visitor.visit_non_terminal_symbol(this, rule)
    }
    Sequence(seq) => {
      visitor.visit_sequence(this, seq)
    }
    Choice(choices) => {
      visitor.visit_choice(this, choices)
    }
    ZeroOrMore(child) => {
      visitor.visit_zero_or_more(this, child)
    }
    OneOrMore(child) => {
      visitor.visit_one_or_more(this, child)
    }
    ZeroOrOne(child) => {
      visitor.visit_optional(this, child)
    }
    NotPredicate(child) => {
      visitor.visit_not_predicate(this, child)
    }
    AndPredicate(child) => {
      visitor.visit_and_predicate(this, child)
    }
    CharacterClass(char_class) => {
      visitor.visit_character_class(this, char_class)
    }
    SemanticAction(child, action) => {
      visitor.visit_semantic_action(this, child, action)
    }
    TypeAscription(child, ty) => {
      visitor.visit_type_ascription(this, child, ty)
    }
    SpannedExpr(child) => {
      visitor.visit_spanned_expr(this, child)
    }
  }
}

pub fn walk_exprs<R, V: ?Sized>(visitor: &mut V, exprs: Vec<usize>) -> Vec<R> where
  V: Visitor<R>
{
  exprs.into_iter().map(|expr| visitor.visit_expr(expr)).collect()
}
