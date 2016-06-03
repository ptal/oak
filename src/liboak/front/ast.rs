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

pub struct Grammar
{
  pub name: Ident,
  pub rules: Vec<Rule>,
  pub exprs: Vec<Expression>,
  pub exprs_info: Vec<ExpressionInfo>,
  pub rust_items: Vec<RItem>,
  pub attributes: Vec<Attribute>
}

impl Grammar
{
  pub fn new(grammar_name: Ident) -> Grammar {
    Grammar {
      name: grammar_name,
      rules: vec![],
      exprs: vec![],
      exprs_info: vec![],
      rust_items: vec![],
      attributes: vec![]
    }
  }

  pub fn alloc_expr(&mut self, lo: BytePos, hi: BytePos, expr: Expression) -> usize {
    let expr_idx = self.exprs.len();
    self.exprs.push(expr);
    self.exprs_info.push(ExpressionInfo::spanned(lo, hi));
    expr_idx
  }

  pub fn push_rule(&mut self, name: SpannedIdent, attributes: Vec<Attribute>, def: usize) {
    self.rules.push(Rule::new(name, attributes, def));
  }

  pub fn push_attr(&mut self, attr: Attribute) {
    self.attributes.push(attr);
  }

  pub fn push_rust_item(&mut self, ritem: RItem) {
    self.rust_items.push(ritem);
  }

  pub fn expr_ty(&mut self, expr: usize, ty: TypeAnnotation) {
    self.exprs_info[expr].ty = Some(ty);
  }
}

#[derive(Clone)]
pub struct Rule
{
  pub name: SpannedIdent,
  pub attributes: Vec<Attribute>,
  pub def: usize
}

impl Rule
{
  pub fn new(name: SpannedIdent, attributes: Vec<Attribute>, def: usize) -> Rule {
    Rule {
      name: name,
      attributes: attributes,
      def: def
    }
  }
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

#[derive(Clone, Copy)]
pub enum TypeAnnotation {
  Invisible,
  Unit
}

// Implicitly typed expression.
#[derive(Clone)]
pub struct ExpressionInfo
{
  pub span: Span,
  pub ty: Option<TypeAnnotation>
}

impl ExpressionInfo
{
  fn spanned(lo: BytePos, hi: BytePos) -> ExpressionInfo {
    ExpressionInfo {
      span: mk_sp(lo, hi),
      ty: None
    }
  }
}
