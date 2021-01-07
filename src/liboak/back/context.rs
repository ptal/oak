
// Copyright 2016 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub use back::continuation::*;
use back::name_factory::*;
use back::compiler::ExprCompilerFn;
use back::compiler::rtype::*;
use back::compiler::{recognizer_compiler, parser_compiler};
use back::compiler::value::*;

use quote::quote;
use syn::parse_quote;

pub struct Context<'a>
{
  grammar: &'a TGrammar,
  closures: Vec<syn::Stmt>,
  name_factory: NameFactory,
  free_variables: Vec<Ident>,
  mut_ref_free_variables: Vec<(Ident, syn::Type)>,
  num_combinators_compiled: usize
}

impl<'a> Context<'a>
{
  pub fn new(grammar: &'a TGrammar) -> Self
  {
    Context {
      grammar: grammar,
      closures: vec![],
      name_factory: NameFactory::new(),
      free_variables: vec![],
      mut_ref_free_variables: vec![],
      num_combinators_compiled: 0
    }
  }

  pub fn into_recognizer_function(self, body: syn::Expr, rule: Rule) -> syn::Item {
    let recognizer_fn = recognizer_id(rule.ident());
    self.function(recognizer_fn, true, body, parse_quote!(()))
  }

  pub fn into_parser_alias(self, rule: Rule) -> syn::Item {
    let id = rule.ident();
    let recognizer_fn = recognizer_name(parse_quote!(#id));
    let parser_fn = parser_id(id);
    self.function(parser_fn, false,
      parse_quote!(#recognizer_fn(state)),
      parse_quote!(()))
  }

  pub fn into_parser_function(self, body: syn::Expr, rule: Rule) -> syn::Item {
    let parser_fn = parser_id(rule.ident());
    let ty = TypeCompiler::compile(self.grammar, rule.expr_idx);
    self.function(parser_fn, true, body, ty)
  }

  fn function(self, name: Ident, state_mut: bool, body: syn::Expr, ty: syn::Type) -> syn::Item {
    let state_param = self.state_param(state_mut);
    let stream_ty = self.grammar.stream_type();
    let generics = self.grammar.stream_generics();
    let closures = self.closures;
    parse_quote!(
      #[inline]
      pub fn #name #generics (#state_param) -> oak_runtime::ParseState<#stream_ty, #ty>
      {
        #(#closures)*
        #body
      }
    )
  }

  fn state_param(&self, state_mut: bool) -> syn::FnArg {
    let mut_kw = if state_mut {
      Some(quote!(mut))
    } else {
      None
    };
    let stream_ty = self.grammar.stream_type();
    parse_quote!(#mut_kw state: oak_runtime::ParseState<#stream_ty, ()>)
  }

  pub fn compile(&mut self, compiler: ExprCompilerFn, idx: usize,
    success: syn::Expr, failure: syn::Expr) -> syn::Expr
  {
    let compiler = compiler(&self.grammar, idx);
    compiler.compile_expr(self, Continuation::new(success, failure))
  }

  pub fn compile_success(&mut self, compiler: ExprCompilerFn, idx: usize,
    success: syn::Expr, failure: syn::Expr) -> syn::Expr
  {
    let expr = self.compile(compiler, idx, success, failure);
    self.num_combinators_compiled += 1;
    expr
  }

  pub fn compile_recognizer_expr(&mut self, idx: usize) -> syn::Expr {
    Continuation::new(
      parse_quote!(state),
      parse_quote!(state.failure())
    )
    .compile_success(self, recognizer_compiler, idx)
    .unwrap_success()
  }

  pub fn value_constructor<F>(&mut self,
    expr_idx: usize,
    value_ty: syn::Type,
    value_constructor: F) -> (syn::Expr, Ident) where
   F: FnOnce(Ident, syn::Expr) -> syn::Expr,
  {
    let result_var = self.next_free_var();
    let scope = self.open_scope(expr_idx);
    self.push_mut_ref_fv(result_var.clone(), value_ty);
    let result_value = tuple_value(self.free_variables());
    let body =
      Continuation::new(
        value_constructor(result_var.clone(), result_value),
        parse_quote!(state.failure())
      )
      .compile_success(self, parser_compiler, expr_idx)
      .unwrap_success();
    self.close_scope(scope);
    (body, result_var)
  }

  pub fn do_not_duplicate_success(&self) -> bool {
    self.num_combinators_compiled > 0
  }

  pub fn success_as_closure(&mut self, continuation: Continuation) -> Continuation {
    if self.do_not_duplicate_success() {
      self.num_combinators_compiled = 0;
      let closure_name = self.name_factory.next_closure_name();
      let args = self.closure_args();
      let params = self.closure_params();
      continuation.map_success(|success, _| {
        self.closures.push(parse_quote!(let #closure_name = |#(#params),*| #success;));
        parse_quote!(#closure_name(#(#args),*))
      })
    }
    else {
      continuation
    }
  }

  fn closure_params(&self) -> Vec<syn::FnArg> {
    vec![self.state_param(true)]
      .into_iter()
      .chain(self.mut_ref_free_variables
        .iter().cloned()
        .map(|(var, ty)| parse_quote!(#var: &mut #ty)))
      .chain(self.free_variables
        .iter()
        .map(|var| parse_quote!(#var:_)))
      .collect()
  }

  fn closure_args(&self) -> Vec<syn::Expr> {
    vec![parse_quote!(state)]
      .into_iter()
      .chain(self.mut_ref_free_variables
        .iter().cloned()
        .map(|(var, _)| parse_quote!(&mut #var)))
      .chain(self.free_variables
        .iter()
        .map(|var| parse_quote!(#var)))
      .collect()
  }

  pub fn next_mark_name(&mut self) -> Ident {
    self.name_factory.next_mark_name()
  }

  pub fn next_counter_name(&mut self) -> Ident {
    self.name_factory.next_counter_name()
  }

  pub fn next_branch_failed_name(&mut self) -> Ident {
    self.name_factory.next_branch_failed_name()
  }

  pub fn next_free_var(&mut self) -> Ident {
    self.free_variables.pop().expect("Free variables are all bound.")
  }

  pub fn next_free_var_skip(&mut self, expr_idx: usize) -> Ident {
    let card = self.expr_cardinality(expr_idx);
    let len_fv = self.free_variables.len();
    self.free_variables.remove(len_fv-1-card)
  }

  pub fn free_variables(&self) -> Vec<Ident> {
    self.free_variables.clone()
  }

  pub fn push_mut_ref_fv(&mut self, mut_ref_var: Ident, mut_ref_ty: syn::Type) {
    self.mut_ref_free_variables.push((mut_ref_var,mut_ref_ty));
  }

  pub fn pop_mut_ref_fv(&mut self) {
    self.mut_ref_free_variables.pop()
      .expect("There is no mut ref free variables.");
  }

  pub fn expr_cardinality(&self, expr_idx: usize) -> usize {
    self.grammar[expr_idx].type_cardinality()
  }

  pub fn has_unit_type(&self, expr_idx: usize) -> bool {
    self.grammar[expr_idx].ty == crate::middle::typing::ast::Type::Unit
  }

  pub fn open_scope(&mut self, expr_idx: usize) -> Scope {
    let scope = self.save_scope();
    self.num_combinators_compiled = 0;
    self.mut_ref_free_variables = vec![];
    let cardinality = self.expr_cardinality(expr_idx);
    let free_vars = self.name_factory.fresh_vars(cardinality);
    self.free_variables = free_vars;
    scope
  }

  pub fn close_scope(&mut self, scope: Scope) {
    assert!(self.free_variables.is_empty(),
      "Try to close the scope but all free variables have not been bounded.");
    self.restore_scope(scope);
  }

  pub fn save_scope(&self) -> Scope {
    Scope::new(
      self.num_combinators_compiled,
      self.free_variables.clone(),
      self.mut_ref_free_variables.clone()
    )
  }

  pub fn restore_scope(&mut self, scope: Scope) {
    self.num_combinators_compiled = scope.num_combinators_compiled;
    self.mut_ref_free_variables = scope.mut_ref_free_variables;
    self.free_variables = scope.free_variables;
  }
}

#[derive(Clone)]
pub struct Scope {
  num_combinators_compiled: usize,
  free_variables: Vec<Ident>,
  mut_ref_free_variables: Vec<(Ident, syn::Type)>
}

impl Scope {
  fn new(n: usize, fv: Vec<Ident>, mfv: Vec<(Ident, syn::Type)>) -> Self {
    Scope {
      num_combinators_compiled: n,
      free_variables: fv,
      mut_ref_free_variables: mfv
    }
  }
}
