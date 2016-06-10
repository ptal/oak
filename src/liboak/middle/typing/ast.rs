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
pub use ast::Expression::*;

pub use std::collections::HashMap;
pub use std::cell::RefCell;

use rust;
use middle::typing::ast::EvaluationContext::*;
use middle::typing::ast::ExprTy::*;
use front::ast::FExpressionInfo

type TGrammar = Grammar<ExpressionInfo>;

impl TGrammar
{
  pub fn push_expr(&mut self, expr: Expression, expr_info: FExpressionInfo) {
    self.push_expr_info(&expr, expr_info.span, expr_info.ty);
    self.exprs.push(expr);
  }

  fn push_expr_info(&self, expr: &Expression, span: Span,
    ty: Option<TypeAnnotation>)
  {
    let expr_info =
      match ty {
        Some(TypeAnnotation::Invisible) => {
          ExpressionInfo::invisible(span)
        }
        Some(TypeAnnotation::Unit) => {
          ExpressionInfo::unit(span)
        }
        None => {
          self.infer_type(expr, span)
        }
      };
    self.exprs_info.push(expr_info);
  }

  fn infer_type(&self, expr: &Expression, span: Span) -> ExpressionInfo {
    match expr {
      &StrLiteral(_)
    | &NotPredicate(_)
    | &AndPredicate(_) => ExpressionInfo::invisible(span),
      &AnySingleChar
    | &CharacterClass(_)
    | &NonTerminalSymbol(_)
    | &ZeroOrMore(_)
    | &OneOrMore(_)
    | &Optional(_)
    | &Choice(_) => ExpressionInfo::new(span, Identity),
      &Sequence(seq) => ExpressionInfo::new(span, Tuple(seq.clone())),
      &SemanticAction(_, ident) => {
        match grammar.rust_functions[&ident].node {
          &rust::ItemKind::Fn(ref decl,..) => {
            ExpressionInfo::new(span, Action(decl.output.clone()))
          },
          _ => {
            self.span_err(span, format!(
              "Only function items are currently allowed in semantic actions."));
            ExpressionInfo::unit(span)
          }
        }
      }
    }
  }
}

// Explicitly typed expression.
pub struct ExpressionInfo
{
  pub span: Span,
  pub invisible: bool,
  pub ty: ExprTy,
  pub context: EvaluationContext
}

impl ItemSpan for ExpressionInfo {
  fn span(&self) -> Span {
    self.span
  }
}

impl ExpressionInfo
{
  pub fn new(sp: Span, ty: ExprTy) -> ExpressionInfo {
    ExpressionInfo {
      span: sp,
      invisible: false,
      ty: ty,
      context: UnValued
    }
  }

  pub fn unit(sp: Span) -> ExpressionInfo {
    ExpressionInfo::new(sp, ExprTy::unit())
  }

  pub fn invisible(sp: Span) -> ExpressionInfo {
    let mut expr_info = ExpressionInfo::new(sp, ExprTy::unit());
    expr_info.invisible = true;
    expr_info
  }

  // pub fn is_forwading_type(&self) -> bool {
  //   match self.node {
  //     NonTerminalSymbol(_) => true,
  //     Choice(_) => true,
  //     _ => self.ty.borrow().is_projection()
  //   }
  // }

  pub fn to_unit_type(&mut self) {
    self.ty = ExprTy::unit();
  }

  pub fn to_invisible_type(&mut self) {
    self.invisible = true;
    self.to_unit_type();
  }

  pub fn to_tuple_type(&mut self, indexes: Vec<usize>) {
    self.ty = Tuple(indexes);
  }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum EvaluationContext
{
  UnValued,
  Both
}

impl EvaluationContext
{
  pub fn merge(self, other: EvaluationContext) -> EvaluationContext {
    if self != other { Both }
    else { self }
  }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ExprTy
{
  /// The type of the expression is given with a trivial mapping between expressions and types.
  /// For example, `e?` has type `Option<T>` if the type of `e` is `T`.
  Identity,
  /// `Tuple(vec![])` is the unit type.
  /// `Tuple(vec![i])` is a projection of the type of a sub-expression.
  /// `Tuple(vec![i,..,j])` is a tuple for the sub-expressions at index `{i,..,j}`.
  Tuple(Vec<usize>),
  Action(rust::FunctionRetTy)
}

impl ExprTy
{
  pub fn is_unit(&self) -> bool {
    match *self {
      Tuple(ref indexes) => indexes.len() == 0,
      _ => false
    }
  }

  pub fn is_projection(&self) -> bool {
    match *self {
      Tuple(ref indexes) => indexes.len() == 1,
      _ => false
    }
  }

  pub fn unit() -> ExprTy {
    Tuple(vec![])
  }
}
