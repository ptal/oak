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

//! AST of a PEG expression that is shared across all the compiling steps.

pub use identifier::*;
pub use syn::spanned::Spanned;

use std::fmt::{Formatter, Display, Error};

pub use partial::Partial;

pub use middle::typing::ast::IType;
pub use middle::typing::ast::Type;

use middle::analysis::ast::GrammarAttributes;

use std::collections::HashMap;
use std::default::Default;
use std::ops::{Index, IndexMut};

use syn::parse_quote;

pub trait ExprByIndex
{
  fn expr_by_index(&self, index: usize) -> Expression;
}

pub struct Grammar<ExprInfo>
{
  pub start_span: Span,
  pub rules: Vec<Rule>,
  pub exprs: Vec<Expression>,
  pub exprs_info: Vec<ExprInfo>,
  pub stream_alias: syn::ItemType,
  pub rust_functions: HashMap<Ident, syn::ItemFn>,
  pub rust_items: Vec<syn::Item>,
  pub attributes: GrammarAttributes
}

impl<ExprInfo> Grammar<ExprInfo>
{
  pub fn new(start_span: Span, exprs: Vec<Expression>,
    exprs_info: Vec<ExprInfo>) -> Grammar<ExprInfo>
  {
    Grammar {
      start_span,
      rules: vec![],
      exprs,
      exprs_info,
      stream_alias: parse_quote!(pub type Stream<'a> = StrStream<'a>;),
      rust_functions: HashMap::new(),
      rust_items: vec![],
      attributes: GrammarAttributes::default()
    }
  }

  pub fn find_rule_by_ident(&self, id: &Ident) -> Rule {
    self.rules.iter()
      .find(|r| r.ident().to_string() == id.to_string())
      .expect("Rule ident not registered in the known rules.")
      .clone()
  }

  pub fn expr_index_of_rule(&self, id: &Ident) -> usize {
    self.find_rule_by_ident(id).expr_idx
  }

  pub fn stream_generics(&self) -> syn::Generics {
    self.stream_alias.generics.clone()
  }

  /// Given `type Stream<'a, T, ..> where T: X = MyStream<'a, T, ...>`
  /// We generate functions (similar to) the following one:
  ///   fn parse<'a, T, ..>(stream: MyStream<'a, T, ...>) where T: X { ... }
  /// This function creates the type `MyStream<'a, T, ...>` from the type alias.
  /// The generics parameters are supposed to have the same name when we will generate the function.
  /// We must transform the parameters of the type alias into arguments of the function argument's type.
  pub fn stream_type(&self) -> syn::Type {
    let name = self.stream_alias.ident.clone();
    let (_, ty_generics, _) = self.stream_alias.generics.split_for_impl();
    let stream_ty: syn::Type = parse_quote!(#name #ty_generics);
    stream_ty
  }

  /// The span type of the underlying type is given by the trait's associated type `StreamSpan::Output`.
  pub fn span_type(&self) -> syn::Type {
    let range_ty: syn::Type = self.range_type();
    parse_quote!(<#range_ty as StreamSpan>::Output)
  }

  pub fn range_type(&self) -> syn::Type {
    let stream_ty = self.stream_type();
    parse_quote!(Range<#stream_ty>)
  }
}

impl<ExprInfo> Index<usize> for Grammar<ExprInfo>
{
  type Output = ExprInfo;

  fn index<'a>(&'a self, index: usize) -> &'a Self::Output {
    &self.exprs_info[index]
  }
}

impl<ExprInfo> IndexMut<usize> for Grammar<ExprInfo>
{
  fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut Self::Output {
    &mut self.exprs_info[index]
  }
}

impl<ExprInfo> ExprByIndex for Grammar<ExprInfo>
{
  fn expr_by_index(&self, index: usize) -> Expression {
    self.exprs[index].clone()
  }
}

#[derive(Clone)]
pub struct Rule
{
  pub name: Ident,
  pub expr_idx: usize,
}

impl Rule
{
  pub fn new(name: Ident, expr_idx: usize) -> Rule {
    Rule { name, expr_idx }
  }
}

impl ItemIdent for Rule
{
  fn ident(&self) -> Ident {
    self.name.clone()
  }
}

impl Spanned for Rule
{
  fn span(&self) -> Span {
    self.name.span().clone()
  }
}

#[derive(Clone, Debug)]
pub enum Expression
{
  StrLiteral(String), // "match me"
  AnySingleChar, // .
  CharacterClass(CharacterClassExpr), // [0-9]
  NonTerminalSymbol(Ident), // a_rule
  ExternalNonTerminalSymbol(syn::Path), // RustItem
  Sequence(Vec<usize>), // a_rule next_rule
  Choice(Vec<usize>), // try_this / or_try_this_one
  ZeroOrMore(usize), // expr*
  OneOrMore(usize), // expr+
  ZeroOrOne(usize), // expr?
  NotPredicate(usize), // !expr
  AndPredicate(usize), // &expr
  SemanticAction(usize, bool, syn::Expr), // expr > function, the boolean is true if boxed.
  TypeAscription(usize, IType), // expr:() or expr:(^) or expr:<rust-ty>
  SpannedExpr(usize), // .. expr
  RangeExpr(usize), // ... expr
}

#[derive(Clone, Debug)]
pub struct CharacterClassExpr
{
  pub intervals: Vec<CharacterInterval>
}

impl CharacterClassExpr
{
  pub fn new(intervals: Vec<CharacterInterval>) -> CharacterClassExpr {
    CharacterClassExpr {
      intervals: intervals
    }
  }
}

impl Display for CharacterClassExpr
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    formatter.write_str("[\"")?;
    for interval in &self.intervals {
      interval.fmt(formatter)?;
    }
    formatter.write_str("\"]")
  }
}

#[derive(Clone, Debug)]
pub struct CharacterInterval
{
  pub lo: char,
  pub hi: char
}

impl CharacterInterval
{
  pub fn new(lo: char, hi: char) -> CharacterInterval {
    CharacterInterval {
      lo: lo,
      hi: hi
    }
  }

  pub fn escape_lo(&self) -> String {
    self.lo.escape_default().collect()
  }

  pub fn escape_hi(&self) -> String {
    self.hi.escape_default().collect()
  }
}

impl Display for CharacterInterval
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    if self.lo == self.hi {
      formatter.write_str(self.escape_lo().as_str())
    }
    else {
      formatter.write_fmt(format_args!("{}-{}", self.escape_lo(), self.escape_hi()))
    }
  }
}

pub fn display_path_cycle(path: &Vec<Ident>) -> String {
  let mut path_desc = String::new();
  for rule in path {
    path_desc.extend(format!("{} -> ", rule).chars());
  }
  path_desc.extend(format!("{}", path[0]).chars());
  path_desc
}
