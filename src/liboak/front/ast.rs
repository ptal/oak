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

pub use rust::{Spanned, BytePos, NO_EXPANSION};
pub use ast::*;

pub struct FGrammar
{
  pub name: Ident,
  pub rules: Vec<Rule>,
  pub exprs: Vec<Expression>,
  pub exprs_info: Vec<FExpressionInfo>,
  pub rust_items: Vec<RItem>,
  pub attributes: Vec<Attribute>
}

impl FGrammar
{
  pub fn new(grammar_name: Ident) -> FGrammar {
    FGrammar {
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
    self.exprs_info.push(FExpressionInfo::spanned(lo, hi));
    expr_idx
  }

  pub fn push_rule(&mut self, name: SpannedIdent, def: usize) {
    self.rules.push(Rule::new(name, def));
  }

  pub fn push_attr(&mut self, attr: Attribute) {
    self.attributes.push(attr);
  }

  pub fn push_rust_item(&mut self, ritem: RItem) {
    self.rust_items.push(ritem);
  }
}

// Implicitly typed expression.
#[derive(Clone)]
pub struct FExpressionInfo
{
  pub span: Span
}

impl FExpressionInfo
{
  fn spanned(lo: BytePos, hi: BytePos) -> FExpressionInfo {
    FExpressionInfo {
      span: Span::new(lo,hi,NO_EXPANSION)
    }
  }
}

impl ItemSpan for FExpressionInfo {
  fn span(&self) -> Span {
    self.span
  }
}
