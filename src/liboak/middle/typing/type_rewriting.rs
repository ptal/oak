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

//! Bottom-up inference of tuple types. It will unpack every possible tuple type, therefore type of the form `(T1, (T2, T3))` are flatten into `(T1, T2, T3)`.
//! This transformation always terminates since we checked for recursive type in the `typing::recursive_type` analysis.
//! It also eliminates "forwarding types" of the form `Tuple(vec![idx])` when `idx` points to a tuple type.
//! It is only kept if `idx` points to a identity or action type.

use middle::typing::ast::*;
use middle::typing::ast::IType::*;
use middle::typing::ast::Type::*;

pub struct TypeRewriting;

impl TypeRewriting
{
  pub fn reduce_rec_entry_point(grammar: &IGrammar, rule: &Ident, ty: IType) -> IType {
    match ty {
      Rec(ref r) if r.entry_point() == *rule => Self::reduce_rec(grammar, ty),
      ty => ty
    }
  }

  pub fn reduce_rec(grammar: &IGrammar, ty: IType) -> IType {
    match ty {
      Rec(r) if r.is_polymorphic() => Invisible,
      Rec(r) => {
        for rec_path in &r.path_set {
          for ident in &rec_path.path {
            let ty = grammar.type_of(grammar.expr_index_of_rule(ident));
            match ty {
              Rec(_) => (),
              ty => return ty
            }
          }
        }
        Invisible
      }
      ty => ty
    }
  }

  pub fn reduce_final(ty: IType) -> Type {
    match ty {
      Invisible => Type::Unit,
      Infer => unreachable!("Type must be inferred before reducing to a final type."),
      Rec(_) => unreachable!("Rec types must be removed after the first surface algorithm."),
      External => Type::Rust(syn::parse_str("_").expect("_ (type)")),
      Regular(ty) => ty
    }
  }

  pub fn reduce(grammar: &IGrammar, ty: IType) -> IType {
    assert!(ty != IType::Infer,
      "Only inferred types can be reduced.");
    match ty {
      Regular(ty) => TypeRewriting::reduce_regular(grammar, ty),
      ty => ty
    }
  }

  fn reduce_regular(grammar: &IGrammar, ty: Type) -> IType {
    match ty.clone() {
      Optional(child)
    | List(child) => TypeRewriting::reduce_unary(grammar, child, Regular(ty)),
      Tuple(indexes) => TypeRewriting::reduce_tuple(grammar, indexes),
      ty => Regular(ty)
    }
  }

  fn reduce_unary(grammar: &IGrammar, expr_idx: usize, ty: IType) -> IType {
    let expr_ty = grammar.type_of(expr_idx);
    match expr_ty {
      Invisible => Invisible,
      Rec(r) => Rec(r.to_polymorphic_path()),
      _ => ty
    }
  }

  fn reduce_tuple(grammar: &IGrammar, mut indexes: Vec<usize>) -> IType {
    assert!(indexes.len() > 0,
      "Empty tuple are forbidden: unit type must be represented with `Type::Unit`.");
    indexes = TypeRewriting::tuple_inlining(grammar, indexes);
    let rec_ty = TypeRewriting::tuple_rec(grammar, &indexes);
    if let Some(rec_ty) = rec_ty {
      return rec_ty;
    }
    indexes = TypeRewriting::tuple_invisible(grammar, indexes);
    if indexes.is_empty() {
      return Invisible;
    }
    indexes = TypeRewriting::tuple_unit(grammar, indexes);
    if indexes.is_empty() {
      Regular(Unit)
    }
    else if indexes.len() == 1 {
      grammar.type_of(indexes[0])
    }
    else {
      Regular(Tuple(indexes))
    }
  }

  fn tuple_inlining(grammar: &IGrammar, indexes: Vec<usize>) -> Vec<usize> {
    let mut unpacked_indexes = vec![];
    for idx in indexes {
      match grammar.type_of(idx) {
        Regular(Tuple(indexes)) => unpacked_indexes.extend(indexes.into_iter()),
        _ => unpacked_indexes.push(idx)
      }
    }
    unpacked_indexes
  }

  fn tuple_rec(grammar: &IGrammar, indexes: &Vec<usize>) -> Option<IType> {
    let mut rec_set = RecSet::empty();
    let mut is_polymorphic = false;
    for idx in indexes {
      match grammar.type_of(*idx) {
        Rec(r) => rec_set = rec_set.union(r),
        External => is_polymorphic = true,
        Regular(Unit) => (),
        Regular(_) => is_polymorphic = true,
        _ => ()
      }
    }
    if rec_set.is_empty() {
      None
    }
    else {
      if is_polymorphic {
        rec_set = rec_set.to_polymorphic_path();
      }
      Some(Rec(rec_set))
    }
  }

  fn tuple_invisible(grammar: &IGrammar, indexes: Vec<usize>) -> Vec<usize> {
    indexes.into_iter()
      .filter(|idx| grammar.type_of(*idx) != Invisible)
      .collect()
  }

  fn tuple_unit(grammar: &IGrammar, indexes: Vec<usize>) -> Vec<usize> {
    indexes.into_iter()
      .filter(|idx| grammar.type_of(*idx) != Regular(Unit))
      .collect()
  }

  /// We try to unify the types of each branch of a choice expression.
  /// Let `e1:T1 / ... / en:Tn` be the annotated choice with the types in `tys`, then
  ///   1. We divide the types into two groups: recursive T1..Ti-1 (with `Rec`) and non-recursive Ti..Tn.
  ///   2. If two types in the non-recursive set are different (except for External which unifies with other type), return Err.
  ///   3. If there is a branch with `Rec(Poly)`, return Err.
  ///   4. Otherwise return Ok(T1)
  pub fn reduce_sum(grammar: &IGrammar, tys: Vec<IType>) -> Result<IType, RecSet> {
    assert!(!tys.is_empty(),
      "Only non-empty sum type can be reduced.");
    let mut rec_set = RecSet::empty();
    let mut non_rec_tys = vec![];
    for ty in tys {
      match ty {
        Rec(r) => { rec_set = rec_set.union(r) },
        ty => {
          let contained = non_rec_tys.iter()
            .any(|ty2| ty.syntactic_eq(grammar, ty2));
          if !contained {
            non_rec_tys.push(ty);
          }
        }
      }
    }
    // Contains recursive types that we cannot type.
    if rec_set.is_polymorphic() {
      return Err(rec_set)
    }
    else if non_rec_tys.len() == 1 {
      return Ok(non_rec_tys[0].clone())
    }
    // External type unifies with the other type.
    else if non_rec_tys.len() == 2 {
      if non_rec_tys[0] == External {
        return Ok(non_rec_tys[1].clone())
      }
      else if non_rec_tys[1] == External {
        return Ok(non_rec_tys[0].clone())
      }
    }
    // In case of unit recursive paths only.
    if non_rec_tys.is_empty() {
      Ok(Rec(rec_set))
    }
    // If all types are (^) or ().
    else if non_rec_tys.iter().all(|ty| ty == &Invisible || ty == &Regular(Unit)) {
      Ok(Invisible)
    }
    else {
      Err(rec_set)
    }
  }
}
