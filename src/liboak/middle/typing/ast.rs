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

//! Give a naive type to any expression of the grammar. It also reads the expression type annotations (invisible type `(^)` and the unit type `()`) and modifies the type accordingly. It does not propagate the invisible types, this step is done in the next typing steps.
//! Literals (e.g. `"lit"`) and syntactic predicates (e.g. `&e` and `!e`) are by default invisibles.

use quote::quote;
use syn::parse_quote;

pub use ast::*;
pub use visitor::*;
pub use ast::Expression::*;

use middle::typing::ast::Type::*;
use middle::typing::ast::IType::*;
use middle::analysis::ast::AGrammar;

pub type IGrammar = Grammar<ExprIType>;
pub type TGrammar = Grammar<ExprType>;

impl IGrammar
{
  pub fn from(agrammar: AGrammar) -> IGrammar {
    let exprs_info = agrammar.exprs_info;
    let mut grammar = IGrammar {
      start_span: agrammar.start_span,
      rules: agrammar.rules,
      exprs: agrammar.exprs,
      exprs_info: vec![],
      stream_alias: agrammar.stream_alias,
      rust_functions: agrammar.rust_functions,
      rust_items: agrammar.rust_items,
      attributes: agrammar.attributes
    };
    grammar.exprs_info = exprs_info.into_iter()
      .map(|e| ExprIType::infer(e.span))
      .collect();
    grammar.alloc_span_ty_expr();
    grammar
  }

  fn error_unit_action_type(span: Span) {
    span.unstable().error("a semantic action with type unit (`()`) will never be called as a semantic action is supposed to produce data.\n\
      If this is intentional because the function has side-effects, return a custom type such as `MyUnit` with `type MyUnit = ()`.\n")
    .emit();
  }

  /// If the semantic action is a single identifier, and that we can retrieve a Rust function with the same name, it resolves to the return type of that function.
  /// We try to convert Rust unit type into Oak unit type for better typechecking.
  /// Otherwise, `External` is returned.
  /// If we detect a semantic action with no type or type `()`, we generate an error because this semantic action will never be called.
  pub fn resolve_action_type(&self, span: Span, boxed: bool, action: syn::Expr) -> IType
  {
    match action {
      syn::Expr::Path(expr_path) => {
        if let Some(ident) = expr_path.path.get_ident() {
          if let Some(fun) = self.rust_functions.get(ident) {
            match &fun.sig.output {
              &syn::ReturnType::Default => {
                Self::error_unit_action_type(span);
                Regular(Unit)
              },
              &syn::ReturnType::Type(_, ref ty) => {
                let unit_ty = syn::parse_str("()").expect("unit type");
                let ty = (**ty).clone();
                if ty == unit_ty {
                  Self::error_unit_action_type(span)
                }
                let ty: syn::Type =
                  if boxed { parse_quote!(Box<#ty>) }
                  else { ty };
                Regular(Rust(ty))
              }
            }
          } else { External }
        } else { External }
      }
      _ => External
    }
  }

  pub fn type_of(&self, expr_idx: usize) -> IType {
    self[expr_idx].ty()
  }

  pub fn map_exprs_info(self, exprs_info: Vec<ExprType>) -> TGrammar {
    TGrammar {
      start_span: self.start_span,
      rules: self.rules,
      exprs: self.exprs,
      exprs_info: exprs_info,
      stream_alias: self.stream_alias,
      rust_functions: self.rust_functions,
      rust_items: self.rust_items,
      attributes: self.attributes
    }
  }

  /// The type of an expression `(.. e)` is `(Span, T)` with `T` the type of `e`.
  /// To construct this tuple type, we need to have an index for the type `Span`, which is not registered in the expression tree.
  /// The reason is that a spanned expression is of the form `SpannedExpr(e)`, which has the type `(Span, T)`, and `e` has the type `T`, thus there is no expression that has the type `Span` which is why `Span` does not have an index.
  /// This trick is simply to avoid adding a variant `SpanType` in the `Type` enum.
  fn alloc_span_ty_expr(&mut self) {
    self.exprs.push(Expression::SpannedExpr(0)); // useless, just to keep exprs and exprs_info consistent.
    let span_ty = self.span_type();
    self.exprs_info.push(
      ExpressionInfo::new(span_ty.span().clone(),
        IType::Regular(Type::Rust(span_ty))));
  }

  /// The span type is special: it is always stored in the last position since it is added during typing (in `alloc_span_ty_expr`), and all other expressions are added during parsing.
  pub fn span_ty_idx(&self) -> usize {
    self.exprs_info.len() - 1
  }
}

pub type ExprIType = ExpressionInfo<IType>;
pub type ExprType = ExpressionInfo<Type>;

// Explicitly typed expression.
#[derive(Clone)]
pub struct ExpressionInfo<Ty>
{
  pub span: Span,
  pub ty: Ty
}

impl<Ty> Spanned for ExpressionInfo<Ty> {
  fn span(&self) -> Span {
    self.span
  }
}

impl<Ty> ExpressionInfo<Ty> where
 Ty: Clone
{
  pub fn new(sp: Span, ty: Ty) -> Self {
    ExpressionInfo {
      span: sp,
      ty: ty
    }
  }

  pub fn ty(&self) -> Ty {
    self.ty.clone()
  }
}

impl ExprType {
  pub fn type_cardinality(&self) -> usize {
    self.ty.cardinality()
  }
}

impl ExprIType
{
  pub fn infer(sp: Span) -> Self {
    ExprIType::new(sp, Infer)
  }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RecKind
{
  Unit, // Only unit types are present on the recursive path.
  Poly, // Polymorphic constructors (optional, vec, tuples) are present on the recursive path.
}

#[derive(Clone, Debug)]
pub struct RecPath
{
  kind: RecKind,
  pub path: Vec<Ident>,
}

impl RecPath {
  pub fn new(kind: RecKind, path: Vec<Ident>) -> Self {
    assert!(!path.is_empty(),
      "Only non-empty path are recursive.");
    RecPath {
      kind: kind,
      path: path
    }
  }

  pub fn to_polymorphic_path(self) -> Self {
    RecPath::new(RecKind::Poly, self.path)
  }

  pub fn display(&self) -> String {
    display_path_cycle(&self.path)
  }
}

impl PartialEq for RecPath {
    fn eq(&self, other: &Self) -> bool {
      self.kind == other.kind &&
      self.path.len() == other.path.len() &&
      self.path.iter().zip(other.path.iter()).find(|(a,b)| a.to_string() != b.to_string()).is_none()
    }
}
impl Eq for RecPath {}

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

  pub fn to_polymorphic_path(self) -> Self {
    RecSet {
      path_set: self.path_set.into_iter()
        .map(|path| path.to_polymorphic_path())
        .collect()
    }
  }

  pub fn keep_only_polymorphic_paths(self) -> Self {
    RecSet {
      path_set: self.path_set.into_iter()
        .filter(|path| path.kind == RecKind::Poly)
        .collect()
    }
  }

  pub fn is_polymorphic(&self) -> bool {
    self.path_set.iter().any(|p| p.kind == RecKind::Poly)
  }

  pub fn display(&self) -> String {
    let mut paths = String::new();
    for path in &self.path_set {
      paths.extend(path.display().chars());
      paths.push('\n');
    }
    paths.pop();
    paths
  }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum IType
{
  Infer,
  Rec(RecSet),
  Invisible,
  External, // Type not known to Oak, for instance the result of an external parser call.
  Regular(Type)
}

impl IType
{
  pub fn rec(kind: RecKind, rec_path: Vec<Ident>) -> IType {
    let path = RecPath::new(kind, rec_path);
    Rec(RecSet::new(path))
  }

  pub fn syntactic_eq(&self, grammar: &IGrammar, other: &IType) -> bool {
    match (self.clone(), other.clone()) {
      (Rec(r1), Rec(r2)) => r1 == r2,
      (Invisible, Invisible) => true,
      (External, External) => true,
      (Regular(ty1), Regular(ty2)) => ty1.syntactic_eq(grammar, &ty2),
      _ => false
    }
  }

  pub fn is_unit_kind(&self) -> bool {
    self == &Invisible || self == &Regular(Unit)
  }

  pub fn contains_external(&self, grammar: &IGrammar) -> bool {
    match self.clone() {
      Infer | Rec(_) | Invisible => false,
      External => true,
      Regular(ty) => ty.contains_external(grammar)
    }
  }

  pub fn display(&self, grammar: &IGrammar) -> String {
    match self.clone() {
      Infer => format!("_"),
      Rec(_) => format!("(^)*"),
      Invisible => format!("(^)"),
      External => format!("_"),
      Regular(ty) => ty.display(grammar)
    }
  }
}

impl Default for IType {
  fn default() -> Self { Infer }
}

#[derive(Clone, Debug)]
pub enum Type
{
  Unit,
  Atom,
  Optional(usize),
  List(usize),
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

impl Type
{
  pub fn cardinality(&self) -> usize {
    match *self {
      Unit => 0,
      Atom
    | Optional(_)
    | List(_)
    | Rust(_) => 1,
      Tuple(ref indexes) => indexes.len()
    }
  }

  pub fn syntactic_eq(&self, grammar: &IGrammar, other: &Type) -> bool {
    let syntactic_eq_expr = |e1, e2| {
      let ty1 = grammar.type_of(e1);
      let ty2 = grammar.type_of(e2);
      ty1.syntactic_eq(grammar, &ty2)
    };
    match (self.clone(), other.clone()) {
      (Unit, Unit) => true,
      (Atom, Atom) => true,
      (Optional(e1), Optional(e2))
    | (List(e1), List(e2)) => syntactic_eq_expr(e1, e2),
      (Tuple(exprs1), Tuple(exprs2)) => {
        if exprs1.len() == exprs2.len() {
          for (e1, e2) in exprs1.into_iter().zip(exprs2.into_iter()) {
            if !syntactic_eq_expr(e1, e2) {
              return false;
            }
          }
          true
        }
        else {
          false
        }
      }
      (Rust(_), Rust(_)) => true,
      _ => false
    }
  }

  pub fn display(&self, grammar: &IGrammar) -> String {
    match self.clone() {
      Unit => format!("()"),
      Atom => format!("char"),
      Optional(child) =>
        format!("Option<{}>", grammar.type_of(child).display(grammar)),
      List(child) =>
        format!("Vec<{}>", grammar.type_of(child).display(grammar)),
      Tuple(children) => {
        let mut display = format!("(");
        for child in children {
          display.extend(grammar.type_of(child).display(grammar).chars());
          display.push_str(", ");
        }
        display.pop();
        display.pop();
        display.push(')');
        display
      }
      Rust(rty) => {
        format!("{}", quote!(#rty))
      }
    }
  }

  pub fn contains_external(&self, grammar: &IGrammar) -> bool {
    match self.clone() {
      Unit | Atom | Rust(_) => false,
      Optional(child) | List(child) => {
        let ty = grammar.type_of(child);
        ty.contains_external(grammar)
      }
      Tuple(children) => {
        let tys: Vec<_> = children.into_iter().map(|c| grammar.type_of(c)).collect();
        tys.into_iter().any(|ty| ty.contains_external(grammar))
      }
    }
  }
}
