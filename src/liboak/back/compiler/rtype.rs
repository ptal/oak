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

//! Generates Rust type from Oak type.

use rust;
use rust::{FunctionRetTy, AstBuilder};
use middle::typing::ast::*;
use middle::typing::ast::ExprTy::*;

pub struct TypeCompiler<'a: 'c, 'b: 'a, 'c>
{
  grammar: &'c TGrammar<'a, 'b>
}

impl<'a, 'b, 'c> TypeCompiler<'a, 'b, 'c>
{
  pub fn compile(grammar: &'c TGrammar<'a, 'b>, expr_idx: usize) -> RTy {
    let compiler = TypeCompiler::new(grammar);
    compiler.compile_type(expr_idx)
  }

  fn new(grammar: &'c TGrammar<'a, 'b>) -> TypeCompiler<'a, 'b, 'c> {
    TypeCompiler {
      grammar: grammar
    }
  }

  fn compile_type(&self, expr_idx: usize) -> RTy {
    match self.grammar[expr_idx].ty.clone() {
      Action(rust_ty) => self.action_type(rust_ty),
      Tuple(ref indexes) if indexes.len() == 0 => self.unit_type(),
      Tuple(ref indexes) if indexes.len() == 1 => self.compile_type(indexes[0]),
      Tuple(indexes) => self.tuple_type(expr_idx, indexes),
      Identity => self.identity_type(expr_idx)
    }
  }

  fn action_type(&self, return_ty: FunctionRetTy) -> RTy {
    match return_ty {
      FunctionRetTy::None(_) | FunctionRetTy::Default(_) => self.unit_type(),
      FunctionRetTy::Ty(ty) => ty
    }
  }

  fn unit_type(&self) -> RTy {
    quote_ty!(self.grammar.cx, ())
  }

  fn tuple_type(&self, expr_idx: usize, indexes: Vec<usize>) -> RTy {
    let tys: Vec<_> = indexes.into_iter()
      .map(|idx| self.compile_type(idx))
      .collect();
    let span = self.grammar[expr_idx].span;
    self.grammar.cx.ty(span, rust::TyKind::Tup(tys))
  }

  fn identity_type(&self, expr_idx: usize) -> RTy {
    match self.grammar.expr_by_index(expr_idx).clone() {
      AnySingleChar
    | CharacterClass(_) => self.char_type(),
      ZeroOrMore(expr_idx)
    | OneOrMore(expr_idx) => self.vector_type(expr_idx),
      Optional(expr_idx) => self.option_type(expr_idx),
      expr => panic!(format!("Expression `{:?}` of type `Identity` not recognized.", expr))
    }
  }

  fn char_type(&self) -> RTy {
    quote_ty!(self.grammar.cx, char)
  }

  fn vector_type(&self, expr_idx: usize) -> RTy {
    let ty = self.compile_type(expr_idx);
    quote_ty!(self.grammar.cx, Vec<$ty>)
  }

  fn option_type(&self, expr_idx: usize) -> RTy {
    let ty = self.compile_type(expr_idx);
    quote_ty!(self.grammar.cx, Option<$ty>)
  }
}
