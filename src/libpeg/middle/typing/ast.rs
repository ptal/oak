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
pub use identifier::*;

pub use rust::{ExtCtxt, Span, Spanned, SpannedIdent};

pub use std::collections::HashMap;
pub use std::rc::Rc;
pub use std::cell::RefCell;

use rust;
use middle::typing::ast::ExpressionTypeVersion::*;
use middle::typing::ast::ExpressionType::*;
use middle::typing::ast::NamedExpressionType::*;

pub struct Grammar
{
  pub name: Ident,
  pub rules: HashMap<Ident, Rule>,
  pub named_types: HashMap<Ident, NamedExpressionType>,
  pub rust_items: HashMap<Ident, rust::P<rust::Item>>,
  pub attributes: GrammarAttributes
}

pub struct Rule
{
  pub name: SpannedIdent,
  pub def: Box<Expression>,
  pub attributes: RuleAttributes
}

#[deriving(Clone)]
pub enum ExpressionTypeVersion
{
  Typed,
  UnTyped,
  Both
}

// Explicitly typed expression.
#[deriving(Clone)]
pub struct Expression
{
  pub span: Span,
  pub node: ExpressionNode,
  pub ty: PTy,
  pub version: ExpressionTypeVersion
}

impl Expression
{
  pub fn new(sp: Span, node: ExpressionNode, ty: PTy) -> Expression
  {
    Expression {
      span: sp,
      node: node,
      ty: ty,
      version: Both
    }
  }
}

pub type ExpressionNode = Expression_<Expression>;

// Type pointer. The types are a DAG structure because type loops are guarded
// by the RuleTypePlaceholder or RuleTypeName constructors: types are indirectly
// referenced through a ident.
// The type can be replaced during the inlining or propagation and that's why
// we use a RefCell. Note that a RefCell has a unique owner or is guarded by
// a Rc (proof by induction).
pub type PTy = RefCell<Rc<ExpressionType>>;

pub fn make_pty(expr: ExpressionType) -> PTy
{
  RefCell::new(Rc::new(expr))
}

#[deriving(Clone)]
pub enum ExpressionType
{
  Character,
  Unit,
  UnitPropagate,
  RuleTypePlaceholder(Ident),
  RuleTypeName(Ident),
  Vector(PTy),
  Tuple(Vec<PTy>),
  OptionalTy(PTy),
  UnnamedSum(Vec<PTy>),
  Action(rust::FunctionRetTy)
}

#[deriving(Clone)]
pub enum NamedExpressionType
{
  Struct(String, Vec<(String, PTy)>),
  StructTuple(String, Vec<PTy>),
  Sum(String, Vec<(String, PTy)>),
  TypeAlias(String, PTy)
}

impl Rule
{
  pub fn is_inline(&self) -> bool
  {
    match self.attributes.ty.style {
      Inline(_) => true,
      _ => false
    }
  }
}

impl ExpressionType
{
  pub fn must_propagate(&self) -> bool
  {
    match self {
      &UnitPropagate => true,
      _ => false
    }
  }

  pub fn is_unit(&self) -> bool
  {
    match self {
      &UnitPropagate => true,
      &Unit => true,
      _ => false
    }
  }

  pub fn is_type_ph(&self) -> bool
  {
    match self {
      &RuleTypePlaceholder(_) => true,
      _ => false
    }
  }

  pub fn ph_ident(&self) -> Ident
  {
    match self {
      &RuleTypePlaceholder(ref ident) => ident.clone(),
      _ => panic!("Cannot extract ident of `RuleTypePlaceholder` from `ExpressionType`.")
    }
  }
}
