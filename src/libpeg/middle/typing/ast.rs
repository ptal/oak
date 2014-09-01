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
pub use middle::attribute::ast::{
  StrLiteral, AnySingleChar, NonTerminalSymbol, Sequence,
  Choice, ZeroOrMore, OneOrMore, Optional, NotPredicate,
  AndPredicate, CharacterClass};

pub use middle::attribute::attribute::*;

pub use rust::{ExtCtxt, Span, Spanned, SpannedIdent};
pub use std::collections::hashmap::HashMap;
pub use identifier::*;

pub use std::rc::Rc;
pub use std::cell::RefCell;

pub struct Grammar
{
  pub name: Ident,
  pub rules: HashMap<Ident, Rule>,
  pub named_types: HashMap<Ident, NamedExpressionType>,
  pub attributes: GrammarAttributes
}

pub struct Rule
{
  pub name: SpannedIdent,
  pub def: Box<Expression>,
  pub attributes: RuleAttributes
}

// Explicitly typed expression.
#[deriving(Clone)]
pub struct Expression
{
  pub span: Span,
  pub node: ExpressionNode,
  pub ty: PTy
}

pub type ExpressionNode = Expression_<Expression>;

// Pointer to type.
pub type PTy = Rc<RefCell<ExpressionType>>;

pub fn make_pty(expr: ExpressionType) -> PTy
{
  Rc::new(RefCell::new(expr))
}

#[deriving(Clone)]
pub enum ExpressionType
{
  Character,
  Unit,
  UnitPropagate,
  RuleTypePlaceholder(Ident),
  // RuleTypeName(Ident),
  Vector(PTy),
  Tuple(Vec<PTy>),
  OptionalTy(PTy),
  UnnamedSum(Vec<PTy>)
}

// #[deriving(Clone)]
// pub enum RuleType
// {
//   InlineTy(Rc<ExpressionType>),
//   NewTy(Box<NamedExpressionType>)
// }

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
  pub fn propagate(&self, self_rc: PTy, 
    f: |PTy| -> PTy) -> PTy
  {
    match self {
      &UnitPropagate => self_rc,
      _ => f(self_rc)
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
      _ => fail!("Cannot extract ident of `RuleTypePlaceholder` from `ExpressionType`.")
    }
  }
}
