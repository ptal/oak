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

//! Bottom-up inference of tuple types. It will unpack every possible tuple type, therefore type of the form `(T1, (T2, T3))` are flatten into `(T1, T2, T3)`. This transformation always terminates since we checked for recursive type in the `typing::recursive_type` analysis. It also eliminates "forwarding types" of the form `Tuple(vec![idx])` when `idx` points to a tuple type. It is only kept if `idx` points to a identity or action type.

use middle::typing::ast::*;
use middle::typing::ast::IType::*;
use middle::typing::ast::Type::*;

pub struct TypeRewriting;

impl TypeRewriting
{
  pub fn reduce_rec(rule: Ident, ty: IType) -> IType {
    match ty {
      Rec(ref r) if r[0] == rule => Invisible,
      ty => ty
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

  pub fn final_reduce(ty: IType) -> Type {
    match ty {
      Rec(_) | Invisible => Type::Unit,
      Infer => unreachable!("Type must be inferred before reducing to a final type."),
      Regular(ty) => ty
    }
  }

  fn reduce_regular(grammar: &IGrammar, ty: Type) -> IType {
    match ty.clone() {
      Optional(child)
    | List(child) => TypeRewriting::reduce_or(grammar, child, Regular(ty)),
      Tuple(indexes) => TypeRewriting::reduce_tuple(grammar, indexes),
      ty => Regular(ty)
    }
  }

  fn reduce_or(grammar: &IGrammar, expr_idx: usize, ty: IType) -> IType {
    let child_ty = grammar.type_of(expr_idx);
    match child_ty.clone() {
      Invisible
    | Rec(_) => child_ty,
      _ => ty
    }
  }

  fn reduce_tuple(grammar: &IGrammar, mut indexes: Vec<usize>) -> IType {
    assert!(indexes.len() > 0,
      "Empty tuple are forbidden: unit type must be represented with `Type::Unit`.");
    indexes = TypeRewriting::tuple_inlining(grammar, indexes);
    indexes = TypeRewriting::tuple_rec(grammar, indexes);
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

  fn tuple_rec(grammar: &IGrammar, indexes: Vec<usize>) -> Vec<usize> {
    for idx in indexes.clone() {
      if let Rec(_) = grammar.type_of(idx) {
        return vec![idx]
      }
    }
    indexes
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
}
