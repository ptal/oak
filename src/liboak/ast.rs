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

use rust;
use std::fmt::{Formatter, Write, Display, Error};

pub type RTy = rust::P<rust::Ty>;
pub type RExpr = rust::P<rust::Expr>;
pub type RItem = rust::P<rust::Item>;

pub use rust::{ExtCtxt, Attribute, SpannedIdent};
pub use monad::partial::Partial;

use middle::analysis::ast::GrammarAttributes;

use std::collections::HashMap;
use std::default::Default;

pub struct Grammar<'cx, ExprInfo>
{
  pub cx: &'cx ExtCtxt<'cx>,
  pub name: Ident,
  pub rules: HashMap<Ident, Rule>,
  pub exprs: Vec<Expression>,
  pub exprs_info: Vec<ExprInfo>,
  pub rust_functions: HashMap<Ident, RItem>,
  pub rust_items: Vec<RItem>,
  pub attributes: GrammarAttributes
}

impl<'cx, ExprInfo> Grammar<'cx, ExprInfo>
{
  pub fn new(cx: &'cx ExtCtxt<'cx>, name: Ident, exprs: Vec<Expression>,
    exprs_info: Vec<ExprInfo>) -> Grammar<'cx, ExprInfo>
  {
    Grammar {
      cx: cx,
      name: name,
      rules: HashMap::new(),
      exprs: exprs,
      exprs_info: exprs_info,
      rust_functions: HashMap::new(),
      rust_items: vec![],
      attributes: GrammarAttributes::default()
    }
  }

  pub fn info_by_index<'a>(&'a self, index: usize) -> &'a ExprInfo {
    &self.exprs_info[index]
  }

  pub fn warn(&self, msg: String) {
    self.cx.parse_sess.span_diagnostic.warn(msg.as_str());
  }

  pub fn multi_locations_err(&self, sp_err: Span, err: String, sp_note: Span, note: String) {
    self.cx
      .struct_span_err(sp_err, err.as_str())
      .span_note(sp_note, note.as_str())
      .emit();
  }

  pub fn span_err(&self, span: Span, msg: String) {
    self.cx.span_err(span, msg.as_str());
  }
}

impl<'cx, ExprInfo> Grammar<'cx, ExprInfo> where
 ExprInfo: ItemSpan
{
  pub fn expr_err(&self, expr_idx: usize, msg: String) {
    let expr_info = self.info_by_index(expr_idx);
    self.span_err(expr_info.span(), msg);
  }
}

impl<'cx, ExprInfo> ExprByIndex for Grammar<'cx, ExprInfo>
{
  fn expr_by_index<'a>(&'a self, index: usize) -> &'a Expression {
    &self.exprs[index]
  }
}

pub struct Rule
{
  pub name: SpannedIdent,
  pub def: usize,
}

impl Rule
{
  pub fn new(name: SpannedIdent, def: usize) -> Rule {
    Rule{
      name: name,
      def: def
    }
  }
}

impl ItemIdent for Rule
{
  fn ident(&self) -> Ident {
    self.name.node.clone()
  }
}

impl ItemSpan for Rule
{
  fn span(&self) -> Span {
    self.name.span.clone()
  }
}


#[derive(Clone, Debug)]
pub enum Expression
{
  StrLiteral(String), // "match me"
  AnySingleChar, // .
  CharacterClass(CharacterClassExpr), // [0-9]
  NonTerminalSymbol(Ident), // a_rule
  Sequence(Vec<usize>), // a_rule next_rule
  Choice(Vec<usize>), // try_this / or_try_this_one
  ZeroOrMore(usize), // space*
  OneOrMore(usize), // space+
  Optional(usize), // space?
  NotPredicate(usize), // !space
  AndPredicate(usize), // &space
  SemanticAction(usize, Ident) // rule > function
}

#[derive(Clone, Debug)]
pub struct CharacterClassExpr
{
  pub intervals: Vec<CharacterInterval>
}

impl CharacterClassExpr
{
  pub fn new(intervals: Vec<CharacterInterval>) -> CharacterClassExpr {
    CharacterClassExpr {
      intervals: intervals
    }
  }
}

impl Display for CharacterClassExpr
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    try!(formatter.write_str("[\""));
    for interval in &self.intervals {
      try!(interval.fmt(formatter));
    }
    formatter.write_str("\"]")
  }
}

#[derive(Clone, Debug)]
pub struct CharacterInterval
{
  pub lo: char,
  pub hi: char
}

impl CharacterInterval
{
  pub fn new(lo: char, hi: char) -> CharacterInterval {
    CharacterInterval {
      lo: lo,
      hi: hi
    }
  }
}

impl Display for CharacterInterval
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    if self.lo == self.hi {
      formatter.write_char(self.lo)
    }
    else {
      formatter.write_fmt(format_args!("{}-{}", self.lo, self.hi))
    }
  }
}

pub trait ExprByIndex
{
  fn expr_by_index<'a>(&'a self, index: usize) -> &'a Expression;
}

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
  use self::Expression::*;
  match visitor.expr_by_index(parent).clone() {
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

