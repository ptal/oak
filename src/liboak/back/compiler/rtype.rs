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

use middle::typing::ast::*;
use middle::typing::ast::Type::*;
use syn::parse_quote;

pub struct TypeCompiler<'a>
{
  grammar: &'a TGrammar
}

impl<'a> TypeCompiler<'a>
{
  pub fn compile(grammar: &'a TGrammar, expr_idx: usize) -> syn::Type {
    let compiler = TypeCompiler::new(grammar);
    compiler.compile_type(expr_idx)
  }

  fn new(grammar: &'a TGrammar) -> TypeCompiler<'a> {
    TypeCompiler { grammar }
  }

  fn compile_type(&self, expr_idx: usize) -> syn::Type {
    match self.grammar[expr_idx].ty.clone() {
      Unit => Self::unit_type(),
      Atom => self.atom_type(),
      List(expr_idx) => self.list_type(expr_idx),
      Optional(expr_idx) => self.optional_type(expr_idx),
      Rust(rust_ty) => rust_ty,
      Tuple(indexes) => self.tuple_type(indexes),
    }
  }

  pub fn unit_type() -> syn::Type {
    parse_quote!(())
  }

  fn tuple_type(&self, indexes: Vec<usize>) -> syn::Type {
    let tys: Vec<_> = indexes.into_iter()
      .map(|idx| self.compile_type(idx))
      .collect();
    parse_quote!((#(#tys),*))
  }

  fn atom_type(&self) -> syn::Type {
    parse_quote!(char)
  }

  fn list_type(&self, expr_idx: usize) -> syn::Type {
    let ty = self.compile_type(expr_idx);
    parse_quote!(Vec<#ty>)
  }

  fn optional_type(&self, expr_idx: usize) -> syn::Type {
    let ty = self.compile_type(expr_idx);
    parse_quote!(Option<#ty>)
  }
}
