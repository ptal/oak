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

//! Give a naive type to any expression of the grammar. It also reads the expression type annotations (invisible type `(^)` and the unit type `()`) and modifies the type accordingly. It does not propagate the invisible types, this step is done in `typing::bottom_up_unit`.
//! Literals (e.g. `"lit"`) and syntactic predicates (e.g. `&e` and `!e`) are by default invisibles.

pub use ast::*;
pub use visitor::*;
pub use ast::Expression::*;

use rust;
use middle::typing::ast::Type::*;
use middle::typing::ast::IType::*;
use middle::analysis::ast::AGrammar;

pub type IGrammar<'a, 'b> = Grammar<'a, 'b, ExprIType>;
pub type TGrammar<'a, 'b> = Grammar<'a, 'b, ExprType>;

impl<'a, 'b> IGrammar<'a, 'b>
{
  pub fn from(agrammar: AGrammar<'a, 'b>) -> IGrammar<'a, 'b> {
    let exprs_info = agrammar.exprs_info;
    let mut grammar = IGrammar {
      cx: agrammar.cx,
      name: agrammar.name,
      rules: agrammar.rules,
      exprs: agrammar.exprs,
      exprs_info: vec![],
      rust_functions: agrammar.rust_functions,
      rust_items: agrammar.rust_items,
      attributes: agrammar.attributes
    };
    grammar.exprs_info = exprs_info.into_iter()
      .map(|e| ExprIType::infer(e.span))
      .collect();
    grammar
  }

  pub fn action_type(&self, expr_idx: usize, action: Ident) -> IType
  {
    match self.rust_functions[&action].node {
      rust::ItemKind::Fn(ref decl,..) => {
        Regular(Action(decl.output.clone()))
      },
      _ => {
        self.span_err(self[expr_idx].span, format!(
          "Only function items are currently allowed in semantic actions."));
        Regular(Unit)
      }
    }
  }

  pub fn type_of(&self, expr_idx: usize) -> IType {
    self[expr_idx].ty()
  }

  pub fn map_exprs_info(self, exprs_info: Vec<ExprType>) -> TGrammar<'a, 'b> {
    TGrammar {
      cx: self.cx,
      name: self.name,
      rules: self.rules,
      exprs: self.exprs,
      exprs_info: exprs_info,
      rust_functions: self.rust_functions,
      rust_items: self.rust_items,
      attributes: self.attributes
    }
  }
}

pub type ExprIType = ExpressionInfo<IType>;
pub type ExprType = ExpressionInfo<Type>;

// Explicitly typed expression.
#[derive(Clone)]
pub struct ExpressionInfo<Ty>
{
  pub span: Span,
  pub ty: Ty
}

impl<Ty> ItemSpan for ExpressionInfo<Ty> {
  fn span(&self) -> Span {
    self.span
  }
}

impl<Ty> ExpressionInfo<Ty> where
 Ty: Clone
{
  pub fn new(sp: Span, ty: Ty) -> Self {
    ExpressionInfo {
      span: sp,
      ty: ty
    }
  }

  pub fn ty(&self) -> Ty {
    self.ty.clone()
  }
}

impl ExprType {
  pub fn type_cardinality(&self) -> usize {
    self.ty.cardinality()
  }
}

impl ExprIType
{
  pub fn infer(sp: Span) -> Self {
    ExprIType::new(sp, Infer)
  }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum IType
{
  Infer,
  Rec(Vec<Ident>),
  Invisible,
  Regular(Type)
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Type
{
  Unit,
  Atom,
  Optional(usize),
  List(usize),
  /// `Tuple(vec![i,..,j])` is a tuple with the types of the sub-expressions at index `{i,..,j}`.
  /// Precondition: Tuple size >= 2.
  Tuple(Vec<usize>),
  Action(rust::FunctionRetTy)
}

impl Type
{
  pub fn cardinality(&self) -> usize {
    match *self {
      Unit => 0,
      Atom
    | Optional(_)
    | List(_)
    | Action(_) => 1,
      Tuple(ref indexes) => indexes.len()
    }
  }
}
