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

// use rust;
use std::fmt::{Formatter, Display, Error};

// pub type RTy = rust::P<rust::Ty>;
// pub type RExpr = rust::P<rust::Expr>;
// pub type RItem = rust::P<rust::Item>;
// pub type RStmt = Option<rust::Stmt>;
// pub type RPat = rust::P<rust::Pat>;
// pub type RArg = rust::Arg;

// pub use rust::{ExtCtxt, Attribute};
pub use partial::Partial;

pub use ast::IType::*;
pub use ast::Type::*;

// use middle::analysis::ast::GrammarAttributes;

use std::collections::HashMap;
use std::default::Default;
use std::ops::{Index, IndexMut};

use syn::parse_quote;

pub struct GrammarAttributes
{
  pub print_code: PrintLevel,
  pub print_typing: PrintLevel

}

impl Default for GrammarAttributes {
  fn default() -> Self {
    GrammarAttributes {
      print_code: PrintLevel::default(),
      print_typing: PrintLevel::default()
    }
  }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PrintLevel
{
  Debug,
  Show,
  Nothing
}

impl PrintLevel
{
  pub fn merge(self, other: PrintLevel) -> PrintLevel {
    use self::PrintLevel::*;
    match (self, other) {
        (Nothing, Debug)
      | (Show, Debug) => Debug,
      (Nothing, Show) => Show,
      _ => Nothing
    }
  }

  pub fn debug(self) -> bool {
    self == PrintLevel::Debug
  }

  pub fn show(self) -> bool {
    self == PrintLevel::Show
  }
}

impl Default for PrintLevel
{
  fn default() -> PrintLevel {
    PrintLevel::Nothing
  }
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RecKind
{
  Unit,
  Value
}

#[derive(Clone, Debug)]
pub struct RecPath
{
  kind: RecKind,
  pub path: Vec<Ident>,
}

impl PartialEq for RecPath {
    fn eq(&self, other: &Self) -> bool {
      self.kind == other.kind &&
      self.path.len() == other.path.len() &&
      self.path.iter().zip(other.path.iter()).find(|(a,b)| a.to_string() != b.to_string()).is_none()
    }
}
impl Eq for RecPath {}

impl RecPath {
  pub fn new(kind: RecKind, path: Vec<Ident>) -> Self {
    assert!(!path.is_empty(),
      "Only non-empty path are recursive.");
    RecPath {
      kind: kind,
      path: path
    }
  }

  pub fn to_value_kind(self) -> Self {
    RecPath::new(RecKind::Value, self.path)
  }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RecSet
{
  pub path_set: Vec<RecPath>
}

impl RecSet
{
  pub fn new(path: RecPath) -> Self {
     RecSet {
      path_set: vec![path]
    }
  }

  pub fn empty() -> Self {
    RecSet{ path_set: vec![] }
  }

  pub fn is_empty(&self) -> bool {
    self.path_set.is_empty()
  }

  pub fn union(mut self, other: RecSet) -> RecSet {
    for path in other.path_set {
      if !self.path_set.contains(&path) {
        self.path_set.push(path);
      }
    }
    self
  }

  pub fn entry_point(&self) -> Ident {
    assert!(!self.is_empty(),
      "There is no entry point for empty path set.");
    self.path_set[0].path[0].clone()
  }

  pub fn to_value_kind(self) -> Self {
    RecSet {
      path_set: self.path_set.into_iter()
        .map(|path| path.to_value_kind())
        .collect()
    }
  }

  pub fn remove_unit_kind(self) -> Self {
    RecSet {
      path_set: self.path_set.into_iter()
        .filter(|path| path.kind == RecKind::Value)
        .collect()
    }
  }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum IType
{
  Infer,
  Rec(RecSet),
  Invisible,
  Regular(Type)
}

impl IType
{
  pub fn rec(kind: RecKind, rec_path: Vec<Ident>) -> IType {
    let path = RecPath::new(kind, rec_path);
    Rec(RecSet::new(path))
  }

  pub fn is_unit_kind(&self) -> bool {
    self == &Invisible || self == &Regular(Unit)
  }
}

#[derive(Clone, Debug)]
pub enum Type
{
  Unit,
  Atom,
  Optional(usize),
  List(usize),
  // Spanned(usize),
  /// `Tuple(vec![i,..,j])` is a tuple with the types of the sub-expressions at index `{i,..,j}`.
  /// Precondition: Tuple size >= 2.
  Tuple(Vec<usize>),
  Rust(syn::Type)
}

impl PartialEq for Type
{
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Unit, Unit)
    | (Atom, Atom) => true,
      (Optional(e1), Optional(e2))
    | (List(e1), List(e2)) => e1 == e2,
      (Tuple(exprs1), Tuple(exprs2)) => exprs1 == exprs2,
      (Rust(_), Rust(_)) =>
        panic!("Cannot compare `Type::Rust` because `syn::Type` are not comparable."),
      _ => false
    }
  }
}

impl Eq for Type {}

pub trait ExprByIndex
{
  fn expr_by_index(&self, index: usize) -> Expression;
}

pub struct Grammar<ExprInfo>
{
  pub name: Ident,
  pub rules: Vec<Rule>,
  pub exprs: Vec<Expression>,
  pub exprs_info: Vec<ExprInfo>,
  pub stream_alias: syn::Item,
  pub rust_functions: HashMap<Ident, syn::Item>,
  pub rust_items: Vec<syn::Item>,
  pub attributes: GrammarAttributes
}

impl<ExprInfo> Grammar<ExprInfo>
{
  pub fn new(name: Ident, exprs: Vec<Expression>,
    exprs_info: Vec<ExprInfo>) -> Grammar<ExprInfo>
  {
    Grammar {
      name: name,
      rules: vec![],
      exprs: exprs,
      exprs_info: exprs_info,
      stream_alias: parse_quote!(pub type Stream<'a> = StrStream<'a>;),
      rust_functions: HashMap::new(),
      rust_items: vec![],
      attributes: GrammarAttributes::default()
    }
  }

  // pub fn warn(&self, msg: String) {
  //   self.cx.parse_sess.span_diagnostic.warn(msg.as_str());
  // }

  // /// The first element of `errors` will be rendered as an error and the other one as notes.
  // pub fn multi_locations_err(&self, errors: Vec<(Span, String)>) {
  //   assert!(errors.len() > 0, "`errors` must at least contain one element.");
  //   let mut errors_iter = errors.into_iter();
  //   let (span, msg) = errors_iter.next().unwrap();
  //   let mut db = self.cx.struct_span_err(span, msg.as_str());
  //   for (span, msg) in errors_iter {
  //     db.span_note(span, msg.as_str());
  //   }
  //   db.emit();
  // }

  /// The first element of `errors` will be rendered as an error and the other one as notes.
  // pub fn multi_locations_warn(&self, warnings: Vec<(Span, String)>) {
  //   for (span, msg) in warnings {
  //     self.cx.span_warn(span, msg.as_str());
  //   }
  // }

  // pub fn span_warn(&self, span: Span, msg: String) {
  //     self.cx.span_warn(span,msg.as_str());
  // }

  // pub fn span_err(&self, span: Span, msg: String) {
  //   self.cx.span_err(span, msg.as_str());
  // }

  // pub fn span_note(&self, span: Span, msg: String) {
  //   self.cx.parse_sess.span_diagnostic
  //     .span_note_without_error(span, msg.as_str());
  // }

  pub fn find_rule_by_ident(&self, id: Ident) -> Rule {
    self.rules.iter()
      .find(|r| r.ident().to_string() == id.to_string())
      .expect("Rule ident not registered in the known rules.")
      .clone()
  }

  pub fn expr_index_of_rule(&self, id: Ident) -> usize {
    self.find_rule_by_ident(id).expr_idx
  }

  // pub fn stream_generics(&self) -> syn::Generics {
  //   match &self.stream_alias.node {
  //     // The first arg is the type on the right of the type alias declaration.
  //     // `generics` is actually the alias together with all its lifetimes, types and where clause.
  //     &syn::ItemKind::Ty(_, ref generics) => generics.clone(),
  //     _ => unreachable!()
  //   }
  // }

  // Given `type Stream<'a, T, ..> where T: X = MyStream<'a, T, ...>`
  // We generate functions (similar to) the following one:
  //   fn parse<'a, T, ..>(stream: MyStream<'a, T, ...>) where T: X { ... }
  // This function creates the type `MyStream<'a, T, ...>` from the type alias.
  // The generics parameters are supposed to have the same name when we will generate the function.
  // We must transform the parameters of the type alias into arguments of the function argument's type.
  // pub fn stream_type(&self) -> RTy {
  //   match &self.stream_alias.node {
  //     &rust::ItemKind::Ty(ref ty, _) => ty.clone(),
  //     _ => unreachable!()
  //   }
    // let generics = self.stream_generics();
    // let mut generic_args_list = vec![];
    // for param in generics.params.into_iter() {
    //   match param.kind {
    //     rust::GenericParamKind::Lifetime(l) => generic_args_list.push(GenericArg::Lifetime(l.lifetime)),
    //     rust::GenericParamKind::Type{default: ty} => {
    //       let ty = ty.expect("generic parameters of type alias must be explicitly defined (no `_`).").ident;
    //       generic_args_list.push(GenericArg::Type(quote!($ty)));
    //     }
    //   }
    // }
    // let generic_args = GenericArgs::AngleBracketed(
    //   AngleBracketedArgs {
    //     span: rust::DUMMY_SP,
    //     args: generic_args_list,
    //     bindings: vec![],
    //   });

    // let stream_ty = quote!(Stream);
    // if let TyKind::Path(qself, mut path) = stream_ty.node.clone() {
    //   path.segments.last_mut().unwrap().args = Some(P(generic_args));
    //   let ty_path = TyKind::Path(qself, path);
    //   let ty = Ty {
    //     id: stream_ty.id,
    //     node: ty_path,
    //     span: stream_ty.span
    //   };
    //   quote!($ty)
    // } else { unreachable!() }
  // }

  // pub fn span_type(&self) -> RTy {
  //   let stream_ty = self.stream_type();
  //   quote!(<Range<$stream_ty> as StreamSpan>::Output)
  // }
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

impl<ExprInfo> Grammar<ExprInfo> where
 ExprInfo: Spanned
{
  pub fn expr_err(&self, expr_idx: usize, msg: String) {
    self[expr_idx].span().unwrap().error(msg).emit();
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
  pub ty: (Span, IType),
  pub expr_idx: usize,
}

impl Rule
{
  pub fn new(name: Ident, ty: (Span, IType), expr_idx: usize) -> Rule {
    Rule { name, ty, expr_idx }
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
  Sequence(Vec<usize>), // a_rule next_rule
  Choice(Vec<usize>), // try_this / or_try_this_one
  ZeroOrMore(usize), // expr*
  OneOrMore(usize), // expr+
  ZeroOrOne(usize), // expr?
  NotPredicate(usize), // !expr
  AndPredicate(usize), // &expr
  SemanticAction(usize, syn::Expr), // expr > function
  TypeAscription(usize, IType), // expr:() or expr:(^) or expr:<rust-ty>
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
