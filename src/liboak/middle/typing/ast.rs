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

use rust;
use middle::typing::ast::Type::*;
use middle::typing::ast::IType::*;
use middle::analysis::ast::AGrammar;

pub type IGrammar<'a, 'b> = Grammar<'a, 'b, ExprIType>;
pub type TGrammar<'a, 'b> = Grammar<'a, 'b, ExprType>;

impl<'a, 'b> IGrammar<'a, 'b>
{
  pub fn from(agrammar: AGrammar<'a, 'b>) -> IGrammar<'a, 'b> {
    let exprs_info = agrammar.exprs_info;
    let mut grammar = IGrammar {
      cx: agrammar.cx,
      name: agrammar.name,
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

  pub fn action_type(&self, expr_idx: usize, action: Ident) -> IType
  {
    match self.rust_functions[&action].node {
      rust::ItemKind::Fn(ref decl,..) => {
        Regular(Action(decl.output.clone()))
      },
      _ => {
        self.span_err(self[expr_idx].span, format!(
          "Only function items are currently allowed in semantic actions."));
        Regular(Unit)
      }
    }
  }

  pub fn type_of(&self, expr_idx: usize) -> IType {
    self[expr_idx].ty()
  }

  pub fn map_exprs_info(self, exprs_info: Vec<ExprType>) -> TGrammar<'a, 'b> {
    TGrammar {
      cx: self.cx,
      name: self.name,
      rules: self.rules,
      exprs: self.exprs,
      exprs_info: exprs_info,
      stream_alias: self.stream_alias,
      rust_functions: self.rust_functions,
      rust_items: self.rust_items,
      attributes: self.attributes
    }
  }

  /// We allocate a fake spanned expr so we can retrieve its type from `grammar.stream_type()`.
  /// This is because the type of `SpannedExpr(e)` is `(stream_type, e)` but `stream_type` needs an expression index to fit in the type AST.
  fn alloc_span_ty_expr(&mut self) {
    self.exprs.push(Expression::SpannedExpr(0)); // fake, just to keep exprs and exprs_info consistent.
    let span_ty = self.span_type();
    self.exprs_info.push(
      ExpressionInfo::new(span_ty.span.clone(),
        IType::Regular(Type::Action(
          rust::FunctionRetTy::Ty(span_ty)))));
  }

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

impl<Ty> ItemSpan for ExpressionInfo<Ty> {
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
  Unit,
  Value
}

#[derive(Clone, PartialEq, Eq, Debug)]
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

  pub fn to_value_kind(self) -> Self {
    RecPath::new(RecKind::Value, self.path)
  }

  pub fn display(&self) -> String {
    display_path_cycle(&self.path)
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
    self.path_set[0].path[0]
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
      (Regular(ty1), Regular(ty2)) => ty1.syntactic_eq(grammar, &ty2),
      _ => false
    }
  }

  pub fn is_unit_kind(&self) -> bool {
    self == &Invisible || self == &Regular(Unit)
  }

  pub fn display(&self, grammar: &IGrammar) -> String {
    match self.clone() {
      Infer => format!("_"),
      Rec(_) => format!("(^)  *"),
      Invisible => format!("(^)"),
      Regular(ty) => ty.display(grammar)
    }
  }
}

#[derive(Clone, PartialEq, Eq, Debug)]
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
  Action(rust::FunctionRetTy)
}

impl Type
{
  pub fn cardinality(&self) -> usize {
    match *self {
      Unit => 0,
      Atom
    | Optional(_)
    | List(_)
    | Action(_) => 1,
    // | Spanned(_) => 1,
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
    // | (Spanned(e1), Spanned(e2)) => syntactic_eq_expr(e1, e2),
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
      (Action(_), Action(_)) => true,
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
      // Spanned(child) =>
      //   format!("<todo>"), //(<Range<Stream> as StreamSpan>::Output, {})", grammar.type_of(child).display(grammar))
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
      Action(rty) => {
        use rust::FunctionRetTy::*;
        match rty {
          Default(_) => format!("()"),
          Ty(ty) => rust::ty_to_string(&*ty)
        }
      }
    }
  }
}
