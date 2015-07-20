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

pub use middle::attribute::ast::{Expression_, CharacterInterval, CharacterClassExpr};
pub use middle::attribute::ast::Expression_::*;

pub use middle::attribute::attribute::*;

pub use rust::{ExtCtxt, Span, Spanned, SpannedIdent};

pub use std::collections::HashMap;
pub use std::cell::RefCell;

use rust;
use middle::typing::ast::TypingContext::*;
use middle::typing::ast::ExprTy::*;

pub struct Grammar
{
  pub name: Ident,
  pub rules: HashMap<Ident, Rule>,
  pub rust_items: HashMap<Ident, rust::P<rust::Item>>,
  pub attributes: GrammarAttributes
}

pub struct Rule
{
  pub name: SpannedIdent,
  pub def: Box<Expression>
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TypingContext
{
  Typed,
  UnTyped,
  Both
}

impl TypingContext
{
  pub fn merge(self, other: TypingContext) -> TypingContext {
    if self != other { Both }
    else { self }
  }
}

// Explicitly typed expression.
pub struct Expression
{
  pub span: Span,
  pub node: ExpressionNode,
  pub invisible: bool,
  pub ty: RefCell<ExprTy>,
  pub ty_context: TypingContext
}

impl Expression
{
  pub fn new(sp: Span, node: ExpressionNode, ty: ExprTy) -> Expression
  {
    Expression {
      span: sp,
      node: node,
      invisible: false,
      ty: RefCell::new(ty),
      ty_context: Both
    }
  }

  pub fn to_unit_type(&mut self)
  {
    self.ty = RefCell::new(ExprTy::unit());
  }

  pub fn is_by_default_invisible(&self) -> bool {
    match &self.node {
      &StrLiteral(_) | &NotPredicate(_) | &AndPredicate(_) => true,
      _ => false
    }
  }
}

pub type ExpressionNode = Expression_<Expression>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ExprTy
{
  /// The type of the expression is given with a trivial mapping between expressions and types.
  /// For example, `e?` has type `Option<T>` if the type of `e` is `T`.
  Identity,
  /// `Tuple(vec![])` is the unit type.
  /// `Tuple(vec![i])` is a projection of the type of an inner expression.
  /// `Tuple(vec![i,..,j])` is a tuple for the sub-expressions at index `i,..,j`.
  Tuple(Vec<usize>),
  Action(rust::FunctionRetTy)
}

impl ExprTy
{
  pub fn is_unit(&self) -> bool {
    match *self {
      Tuple(ref sub) => sub.len() == 0,
      _ => false
    }
  }

  pub fn unit() -> ExprTy {
    Tuple(vec![])
  }

  pub fn projection(index: usize) -> ExprTy {
    Tuple(vec![index])
  }
}
