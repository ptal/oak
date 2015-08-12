// Copyright 2015 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! AST of a PEG expression that is shared across all the compiling steps.

#![macro_use]

pub use identifier::*;
pub use rust::Span;

use rust;
pub type RTy = rust::P<rust::Ty>;
pub type RExpr = rust::P<rust::Expr>;
pub type RItem = rust::P<rust::Item>;

#[derive(Clone, Debug)]
pub enum Expression_<SubExpr>{
  StrLiteral(String), // "match me"
  AnySingleChar, // .
  CharacterClass(CharacterClassExpr), // [0-9]
  NonTerminalSymbol(Ident), // a_rule
  Sequence(Vec<Box<SubExpr>>), // a_rule next_rule
  Choice(Vec<Box<SubExpr>>), // try_this / or_try_this_one
  ZeroOrMore(Box<SubExpr>), // space*
  OneOrMore(Box<SubExpr>), // space+
  Optional(Box<SubExpr>), // space?
  NotPredicate(Box<SubExpr>), // !space
  AndPredicate(Box<SubExpr>), // &space
  SemanticAction(Box<SubExpr>, Ident) // rule > function
}

#[derive(Clone, Debug)]
pub struct CharacterClassExpr {
  pub intervals: Vec<CharacterInterval>
}

#[derive(Clone, Debug)]
pub struct CharacterInterval {
  pub lo: char,
  pub hi: char
}

pub trait ItemIdent {
  fn ident(&self) -> Ident;
}

pub trait ItemSpan {
  fn span(&self) -> Span;
}

pub trait ExprNode
{
  fn expr_node<'a>(&'a self) -> &'a Expression_<Self>;
}

pub trait Visitor<Node: ExprNode, R>
{
  fn visit_expr(&mut self, expr: &Box<Node>) -> R {
    walk_expr(self, expr)
  }

  fn visit_str_literal(&mut self, _parent: &Box<Node>, _lit: &String) -> R;
  fn visit_non_terminal_symbol(&mut self, _parent: &Box<Node>, _id: Ident) -> R;
  fn visit_character(&mut self, _parent: &Box<Node>) -> R;

  fn visit_any_single_char(&mut self, parent: &Box<Node>) -> R {
    self.visit_character(parent)
  }

  fn visit_character_class(&mut self, parent: &Box<Node>, _expr: &CharacterClassExpr) -> R {
    self.visit_character(parent)
  }

  fn visit_sequence(&mut self, _parent: &Box<Node>, exprs: &Vec<Box<Node>>) -> R;
  fn visit_choice(&mut self, _parent: &Box<Node>, exprs: &Vec<Box<Node>>) -> R;

  fn visit_repeat(&mut self, _parent: &Box<Node>, expr: &Box<Node>) -> R {
    walk_expr(self, expr)
  }

  fn visit_zero_or_more(&mut self, parent: &Box<Node>, expr: &Box<Node>) -> R {
    self.visit_repeat(parent, expr)
  }

  fn visit_one_or_more(&mut self, parent: &Box<Node>, expr: &Box<Node>) -> R {
    self.visit_repeat(parent, expr)
  }

  fn visit_optional(&mut self, _parent: &Box<Node>, expr: &Box<Node>) -> R {
    walk_expr(self, expr)
  }

  fn visit_syntactic_predicate(&mut self, _parent: &Box<Node>, expr: &Box<Node>) -> R {
    walk_expr(self, expr)
  }

  fn visit_not_predicate(&mut self, parent: &Box<Node>, expr: &Box<Node>) -> R {
    self.visit_syntactic_predicate(parent, expr)
  }

  fn visit_and_predicate(&mut self, parent: &Box<Node>, expr: &Box<Node>) -> R {
    self.visit_syntactic_predicate(parent, expr)
  }

  fn visit_semantic_action(&mut self, _parent: &Box<Node>, expr: &Box<Node>, _id: Ident) -> R {
    walk_expr(self, expr)
  }
}

/// We need this macro for factorizing the code since we can not specialize a trait on specific type parameter (we would need to specialize on `()` here).
macro_rules! unit_visitor_impl {
  ($Node:ty, str_literal) => (fn visit_str_literal(&mut self, _parent: &Box<$Node>, _lit: &String) -> () {});
  ($Node:ty, non_terminal) => (fn visit_non_terminal_symbol(&mut self, _parent: &Box<$Node>, _id: Ident) -> () {});
  ($Node:ty, character) => (fn visit_character(&mut self, _parent: &Box<$Node>) -> () {});
  ($Node:ty, sequence) => (
    fn visit_sequence(&mut self, _parent: &Box<$Node>, exprs: &Vec<Box<$Node>>) -> () {
      walk_exprs(self, exprs);
    }
  );
  ($Node:ty, choice) => (
    fn visit_choice(&mut self, _parent: &Box<$Node>, exprs: &Vec<Box<$Node>>) -> () {
      walk_exprs(self, exprs);
    }
  );
}

pub fn walk_expr<Node, R, V: ?Sized>(visitor: &mut V, parent: &Box<Node>) -> R where
  Node: ExprNode,
  V: Visitor<Node, R>
{
  use self::Expression_::*;
  match parent.expr_node() {
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

pub fn walk_exprs<Node, R, V: ?Sized>(visitor: &mut V, exprs: &Vec<Box<Node>>) -> Vec<R> where
  Node: ExprNode,
  V: Visitor<Node, R>
{
  exprs.iter().map(|expr| visitor.visit_expr(expr)).collect()
}

