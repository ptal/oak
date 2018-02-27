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

use rust;
use std::fmt::{Formatter, Display, Error};

pub type RTy = rust::P<rust::Ty>;
pub type RExpr = rust::P<rust::Expr>;
pub type RItem = rust::P<rust::Item>;
pub type RStmt = Option<rust::Stmt>;
pub type RPat = rust::P<rust::Pat>;
pub type RArg = rust::Arg;

pub use rust::{ExtCtxt, Attribute, SpannedIdent};
pub use partial::Partial;

pub use middle::typing::ast::IType;
pub use middle::typing::ast::Type;

use middle::analysis::ast::GrammarAttributes;

use std::collections::HashMap;
use std::default::Default;
use std::ops::{Index, IndexMut};

pub trait ExprByIndex
{
  fn expr_by_index(&self, index: usize) -> Expression;
}

pub struct Grammar<'a, 'b:'a, ExprInfo>
{
  pub cx: &'a ExtCtxt<'b>,
  pub name: Ident,
  pub rules: Vec<Rule>,
  pub exprs: Vec<Expression>,
  pub exprs_info: Vec<ExprInfo>,
  pub stream_alias: RItem,
  pub rust_functions: HashMap<Ident, RItem>,
  pub rust_items: Vec<RItem>,
  pub attributes: GrammarAttributes
}

impl<'a, 'b, ExprInfo> Grammar<'a, 'b, ExprInfo>
{
  pub fn new(cx: &'a ExtCtxt<'b>, name: Ident, exprs: Vec<Expression>,
    exprs_info: Vec<ExprInfo>) -> Grammar<'a, 'b, ExprInfo>
  {
    Grammar {
      cx: cx,
      name: name,
      rules: vec![],
      exprs: exprs,
      exprs_info: exprs_info,
      stream_alias: quote_item!(cx, pub type Stream<'a> = StrStream<'a>;).unwrap(),
      rust_functions: HashMap::new(),
      rust_items: vec![],
      attributes: GrammarAttributes::default()
    }
  }

  pub fn warn(&self, msg: String) {
    self.cx.parse_sess.span_diagnostic.warn(msg.as_str());
  }

  /// The first element of `errors` will be rendered as an error and the other one as notes.
  pub fn multi_locations_err(&self, errors: Vec<(Span, String)>) {
    assert!(errors.len() > 0, "`errors` must at least contain one element.");
    let mut errors_iter = errors.into_iter();
    let (span, msg) = errors_iter.next().unwrap();
    let mut db = self.cx.struct_span_err(span, msg.as_str());
    for (span, msg) in errors_iter {
      db.span_note(span, msg.as_str());
    }
    db.emit();
  }

  /// The first element of `errors` will be rendered as an error and the other one as notes.
  pub fn multi_locations_warn(&self, warnings: Vec<(Span, String)>) {
    for (span, msg) in warnings {
      self.cx.span_warn(span, msg.as_str());
    }
  }

  pub fn span_err(&self, span: Span, msg: String) {
    self.cx.span_err(span, msg.as_str());
  }

  pub fn span_note(&self, span: Span, msg: String) {
    self.cx.parse_sess.span_diagnostic
      .span_note_without_error(span, msg.as_str());
  }

  pub fn find_rule_by_ident(&self, id: Ident) -> Rule {
    self.rules.iter()
      .find(|r| r.ident() == id)
      .expect("Rule ident not registered in the known rules.")
      .clone()
  }

  pub fn expr_index_of_rule(&self, id: Ident) -> usize {
    self.find_rule_by_ident(id).expr_idx
  }

  pub fn stream_generics(&self) -> rust::Generics {
    match &self.stream_alias.node {
      // The first arg is the type on the right of the type alias declaration.
      // `generics` is actually the alias together with all its lifetimes, types and where clause.
      &rust::ItemKind::Ty(_, ref generics) => generics.clone(),
      _ => unreachable!()
    }
  }

  // This function creates the `stream` type from the associated generics in the grammar.
  // We must do all of this because `Generics` and `Ty` are not the same entity in the AST.
  pub fn stream_type(&self) -> RTy {
    use rust::*;
    let generics = self.stream_generics();
    let mut lifetimes = vec![];
    let mut types = vec![];
    for param in generics.params.into_iter() {
      match param {
        rust::GenericParam::Lifetime(l) => lifetimes.push(l.lifetime),
        rust::GenericParam::Type(ty) => {
          let ty = ty.ident;
          types.push(quote_ty!(self.cx, $ty));
        }
      }
    }
    let generics_params = PathParameters::AngleBracketed(
      AngleBracketedParameterData {
        span: rust::DUMMY_SP,
        lifetimes: lifetimes,
        types: types,
        bindings: vec![],
      });

    let stream_ty = quote_ty!(self.cx, Stream);
    if let TyKind::Path(qself, mut path) = stream_ty.node.clone() {
      path.segments.last_mut().unwrap().parameters = Some(P(generics_params));
      let ty_path = TyKind::Path(qself, path);
      let ty = Ty {
        id: stream_ty.id,
        node: ty_path,
        span: stream_ty.span
      };
      quote_ty!(self.cx, $ty)
    } else { unreachable!() }
  }

  pub fn span_type(&self) -> RTy {
    let stream_ty = self.stream_type();
    quote_ty!(self.cx, <Range<$stream_ty> as StreamSpan>::Output)
  }
}

impl<'a, 'b, ExprInfo> Index<usize> for Grammar<'a, 'b, ExprInfo>
{
  type Output = ExprInfo;

  fn index<'c>(&'c self, index: usize) -> &'c Self::Output {
    &self.exprs_info[index]
  }
}

impl<'a, 'b, ExprInfo> IndexMut<usize> for Grammar<'a, 'b, ExprInfo>
{
  fn index_mut<'c>(&'c mut self, index: usize) -> &'c mut Self::Output {
    &mut self.exprs_info[index]
  }
}

impl<'a, 'b, ExprInfo> Grammar<'a, 'b, ExprInfo> where
 ExprInfo: ItemSpan
{
  pub fn expr_err(&self, expr_idx: usize, msg: String) {
    self.span_err(self[expr_idx].span(), msg);
  }
}

impl<'a, 'b, ExprInfo> ExprByIndex for Grammar<'a, 'b, ExprInfo>
{
  fn expr_by_index(&self, index: usize) -> Expression {
    self.exprs[index].clone()
  }
}

#[derive(Clone, Copy)]
pub struct Rule
{
  pub name: SpannedIdent,
  pub expr_idx: usize,
}

impl Rule
{
  pub fn new(name: SpannedIdent, expr_idx: usize) -> Rule {
    Rule{
      name: name,
      expr_idx: expr_idx
    }
  }
}

impl ItemIdent for Rule
{
  fn ident(&self) -> Ident {
    self.name.node.clone()
  }
}

impl ItemSpan for Rule
{
  fn span(&self) -> Span {
    self.name.span.clone()
  }
}

#[derive(Clone, Debug)]
pub enum Expression
{
  StrLiteral(String), // "match me"
  AnySingleChar, // .
  CharacterClass(CharacterClassExpr), // [0-9]
  NonTerminalSymbol(Ident), // a_rule
  Sequence(Vec<usize>), // a_rule next_rule
  Choice(Vec<usize>), // try_this / or_try_this_one
  ZeroOrMore(usize), // expr*
  OneOrMore(usize), // expr+
  ZeroOrOne(usize), // expr?
  NotPredicate(usize), // !expr
  AndPredicate(usize), // &expr
  SemanticAction(usize, Ident), // expr > function
  TypeAscription(usize, IType), // expr -> () or expr -> (^)
  SpannedExpr(usize), // .. expr
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
