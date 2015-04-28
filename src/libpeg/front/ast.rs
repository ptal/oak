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

pub use rust::{SpannedIdent, Spanned, Span, Attribute, BytePos, mk_sp};
pub use identifier::*;
use rust;

pub struct Grammar{
  pub name: Ident,
  pub rules: Vec<Rule>,
  pub rust_items: Vec<rust::P<rust::Item>>,
  pub attributes: Vec<Attribute>
}

#[derive(Clone)]
pub struct Rule{
  pub name: SpannedIdent,
  pub attributes: Vec<Attribute>,
  pub def: Box<Expression>
}

#[derive(Clone)]
pub enum Expression_<SubExpr>{
  StrLiteral(String), // "match me"
  AnySingleChar, // .
  NonTerminalSymbol(Ident), // a_rule
  Sequence(Vec<Box<SubExpr>>), // a_rule next_rule
  Choice(Vec<Box<SubExpr>>), // try_this / or_try_this_one
  ZeroOrMore(Box<SubExpr>), // space*
  OneOrMore(Box<SubExpr>), // space+
  Optional(Box<SubExpr>), // space? - `?` replaced by `$`
  NotPredicate(Box<SubExpr>), // !space
  AndPredicate(Box<SubExpr>), // &space
  CharacterClass(CharacterClassExpr), // [0-9]
  SemanticAction(Box<SubExpr>, Ident) // rule > function
}

#[derive(Clone)]
pub struct CharacterClassExpr {
  pub intervals: Vec<CharacterInterval>
}

#[derive(Clone)]
pub struct CharacterInterval {
  pub lo: char,
  pub hi: char
}

// Implicitly typed expression.
#[derive(Clone)]
pub struct Expression
{
  pub span: Span,
  pub node: ExpressionNode
}

pub fn spanned_expr(lo: BytePos, hi: BytePos, expr: ExpressionNode) -> Box<Expression>
{
  respan_expr(mk_sp(lo, hi), expr)
}

pub fn respan_expr(sp: Span, expr: ExpressionNode) -> Box<Expression>
{
  box Expression {span : sp, node: expr}
}

pub type ExpressionNode = Expression_<Expression>;
