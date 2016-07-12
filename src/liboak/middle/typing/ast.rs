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

pub use std::collections::HashMap;
pub use std::cell::RefCell;

use rust;
use middle::typing::ast::ExprTy::*;
use front::ast::TypeAnnotation;
use middle::analysis::ast::AGrammar;

pub type TGrammar<'a, 'b> = Grammar<'a, 'b, ExpressionInfo>;

impl<'a, 'b> TGrammar<'a, 'b>
{
  pub fn typed_grammar(agrammar: AGrammar<'a, 'b>) -> TGrammar<'a, 'b> {
    let exprs_info = agrammar.exprs_info;
    let mut grammar = TGrammar {
      cx: agrammar.cx,
      name: agrammar.name,
      rules: agrammar.rules,
      exprs: agrammar.exprs,
      exprs_info: vec![],
      rust_functions: agrammar.rust_functions,
      rust_items: agrammar.rust_items,
      attributes: agrammar.attributes
    };
    for (expr_idx, expr_info) in exprs_info.into_iter().enumerate() {
      grammar.push_expr_info(expr_idx, expr_info.span, expr_info.ty);
    };
    grammar
  }

  fn push_expr_info(&mut self, expr_idx: usize, span: Span,
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
          self.infer_type(expr_idx, span)
        }
      };
    self.exprs_info.push(expr_info);
  }

  fn infer_type(&self, expr_idx: usize, span: Span) -> ExpressionInfo {
    match self.expr_by_index(expr_idx) {
      StrLiteral(_)
    | NotPredicate(_)
    | AndPredicate(_) => ExpressionInfo::invisible(span),
      AnySingleChar
    | CharacterClass(_)
    | ZeroOrMore(_)
    | OneOrMore(_)
    | Optional(_) => ExpressionInfo::new(span, Identity),
      Choice(indexes) => ExpressionInfo::new(span, Tuple(vec![indexes[0]])),
      Sequence(indexes) => ExpressionInfo::new(span, Tuple(indexes)),
      NonTerminalSymbol(ident) => {
        let expr_idx = self.expr_index_of_rule(ident);
        ExpressionInfo::new(span, Tuple(vec![expr_idx]))
      }
      SemanticAction(_, ident) => {
        match self.rust_functions[&ident].node {
          rust::ItemKind::Fn(ref decl,..) => {
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

  pub fn print_debug_typing(&self, stage_desc: &str) {

  }

  pub fn print_typing(&self) {

  }
}

// Explicitly typed expression.
pub struct ExpressionInfo
{
  pub span: Span,
  pub invisible: bool,
  pub ty: ExprTy
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
      ty: ty
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

  pub fn is_invisible(&self) -> bool {
    self.invisible
  }

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

  pub fn tuple_indexes(&self) -> Option<Vec<usize>> {
    if let Tuple(ref indexes) = self.ty {
      Some(indexes.clone())
    }
    else {
      None
    }
  }

  pub fn eq_tuple_indexes(&self, indexes: &Vec<usize>) -> bool {
    if let Tuple(ref self_indexes) = self.ty {
      self_indexes == indexes
    }
    else {
      false
    }
  }

  pub fn type_cardinality(&self) -> usize {
    self.ty.cardinality()
  }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ExprTy
{
  /// The expression indicates that a value must be built. For example `e?`, with `e:T`, implies the type `Option<T>` so a new value must be built.
  Identity,
  /// `Tuple(vec![])` is the unit type.
  /// `Tuple(vec![i])` is a projection of the type of a sub-expression.
  /// `Tuple(vec![i,..,j])` is a tuple with the types of the sub-expressions at index `{i,..,j}`.
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

  /// True if the type indicates that a value must be built automatically by the generator.
  pub fn is_value_constructor(&self) -> bool {
    match *self {
      Identity => true,
      Tuple(ref indexes) if indexes.len() > 1 => true,
      _ => false
    }
  }

  pub fn unit() -> ExprTy {
    Tuple(vec![])
  }

  pub fn cardinality(&self) -> usize {
    match *self {
      Identity => 1,
      Action(_) => 1,
      Tuple(ref indexes) => indexes.len()
    }
  }
}
