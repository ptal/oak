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

pub use rust::{SpannedIdent, Spanned, Attribute, BytePos, mk_sp};
pub use ast::*;

pub struct Grammar{
  pub name: Ident,
  pub rules: Vec<Rule>,
  pub rust_items: Vec<RItem>,
  pub attributes: Vec<Attribute>
}

#[derive(Clone)]
pub struct Rule{
  pub name: SpannedIdent,
  pub attributes: Vec<Attribute>,
  pub def: Box<Expression>
}

impl ItemIdent for Rule
{
  fn ident(&self) -> Ident {
    self.name.node
  }
}

impl ItemSpan for Rule
{
  fn span(&self) -> Span {
    self.name.span
  }
}

#[derive(Clone)]
pub enum TypeAnnotation {
  Invisible,
  Unit
}

pub type ExpressionNode = Expression_<Expression>;

// Implicitly typed expression.
#[derive(Clone)]
pub struct Expression
{
  pub span: Span,
  pub node: ExpressionNode,
  pub ty: Option<TypeAnnotation>
}

impl ExprNode for Expression
{
  fn expr_node<'a>(&'a self) -> &'a ExpressionNode
  {
    &self.node
  }
}

pub fn spanned_expr(lo: BytePos, hi: BytePos, expr: ExpressionNode) -> Box<Expression>
{
  respan_expr(mk_sp(lo, hi), expr)
}

pub fn respan_expr(sp: Span, expr: ExpressionNode) -> Box<Expression>
{
  box Expression {span : sp, node: expr, ty: None}
}

