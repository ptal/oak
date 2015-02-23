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
use middle::typing::ast::ExpressionTypeVersion::*;
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
  pub def: Box<Expression>,
  pub attributes: RuleAttributes
}

impl Rule
{
  pub fn is_inline(&self) -> bool
  {
    match self.attributes.ty.style {
      RuleTypeStyle::Inline => true,
      _ => false
    }
  }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ExpressionTypeVersion
{
  Typed,
  UnTyped,
  Both
}

impl ExpressionTypeVersion
{
  pub fn merge(self, other: ExpressionTypeVersion) -> ExpressionTypeVersion {
    if self != other { Both }
    else { self }
  }
}

// Explicitly typed expression.
pub struct Expression
{
  pub span: Span,
  pub node: ExpressionNode,
  pub ty: RefCell<ExprTy>,
  pub version: ExpressionTypeVersion
}

impl Expression
{
  pub fn new(sp: Span, node: ExpressionNode, ty: ExprTy) -> Expression
  {
    Expression {
      span: sp,
      node: node,
      ty: RefCell::new(ty),
      version: Both
    }
  }

  pub fn deref_type(&self, rules: &HashMap<Ident, Rule>) -> ExprTy {
    if let TypeOf(rule_name) = self.ty.borrow().clone() {
      rules.get(&rule_name).unwrap().def.deref_type(rules)
    } else {
      self.ty.borrow().clone()
    }
  }
}

pub type ExpressionNode = Expression_<Expression>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ExprTy
{
  Character,
  Unit,
  UnitPropagate,
  TypeOf(Ident),
  Vector,
  Tuple(Vec<usize>),
  OptionalTy,
  Sum,
  Action(rust::FunctionRetTy)
}

impl ExprTy
{
  pub fn must_propagate(&self) -> bool {
    *self == UnitPropagate
  }

  pub fn is_unit(&self) -> bool {
    match *self {
      UnitPropagate | Unit => true,
      _ => false
    }
  }

  pub fn is_leaf(&self) -> bool {
    match *self {
        UnitPropagate | Unit
      | Character | Action(_) => true,
      _ => false
    }
  }
}
