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

pub use ast::*;
pub use middle::ast::{Grammar_, Rule_, ExprTy};
pub use std::collections::HashMap;
pub use rust::{ExtCtxt, Spanned, SpannedIdent};

use back::ast::FunctionKind::*;

pub type Grammar = Grammar_<Expression>;
pub type Rule = Rule_<Expression>;

pub type ExpressionNode = Expression_<Expression>;

pub struct Expression
{
  pub span: Span,
  pub node: ExpressionNode,
  pub ty: ExprTy,
  pub kind: FunctionKind
}

impl ExprNode for Expression
{
  fn expr_node<'a>(&'a self) -> &'a ExpressionNode
  {
    &self.node
  }
}

impl Expression
{
  pub fn return_type(&self, cx: &ExtCtxt) -> RTy {
    self.kind.to_type(cx)
  }

  pub fn kind(&self) -> FunctionKind {
    self.kind.clone()
  }

  pub fn tuple_indexes(&self) -> Vec<usize> {
    if let ExprTy::Tuple(indexes) = self.ty.clone() {
      indexes
    }
    else {
      panic!("Expected a tuple type for extracting indexes but found {:?}.", self.ty);
    }
  }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FunctionKind
{
  /// Only the recognizer is generated.
  Recognizer,
  /// The parser is an alias to the recognizer. Both functions are generated.
  ParserAlias,
  /// Recognizer and parser are both generated.
  Both(RTy)
}

impl FunctionKind
{
  pub fn is_unit(&self) -> bool {
    match self {
      &Recognizer | &ParserAlias => true,
      _ => false
    }
  }

  pub fn to_type(&self, cx: &ExtCtxt) -> RTy {
    match self.clone() {
      Both(ty) => ty,
      _ => quote_ty!(cx, ())
    }
  }
}
