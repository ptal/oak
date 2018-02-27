
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
use rust;
use rust::AstBuilder;

pub struct Context<'a: 'c, 'b: 'a, 'c>
{
  grammar: &'c TGrammar<'a, 'b>,
  closures: Vec<RStmt>,
  name_factory: NameFactory,
  free_variables: Vec<Ident>,
  mut_ref_free_variables: Vec<(Ident, RTy)>,
  num_combinators_compiled: usize
}

impl<'a, 'b, 'c> Context<'a, 'b, 'c>
{
  pub fn new(grammar: &'c TGrammar<'a, 'b>) -> Self
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

  pub fn into_recognizer_function(self, body: RExpr, rule: Rule) -> RItem {
    let cx = self.cx();
    let recognizer_fn = recognizer_name(cx, rule.ident());
    self.function(recognizer_fn, true, body, quote_ty!(cx, ()))
  }

  pub fn into_parser_alias(self, rule: Rule) -> RItem {
    let cx = self.cx();
    let recognizer_fn = recognizer_name(cx, rule.ident());
    let parser_fn = parser_name(cx, rule.ident());
    self.function(parser_fn, false,
      quote_expr!(cx, $recognizer_fn(state)),
      quote_ty!(cx, ()))
  }

  pub fn into_parser_function(self, body: RExpr, rule: Rule) -> RItem {
    let parser_fn = parser_name(self.cx(), rule.ident());
    let ty = TypeCompiler::compile(self.grammar, rule.expr_idx);
    self.function(parser_fn, true, body, ty)
  }

  fn function(self, name: Ident, state_mut: bool, body: RExpr, ty: RTy) -> RItem {
    let cx = self.cx();
    let state_param = self.state_param(state_mut);
    let stream_ty = self.grammar.stream_type();
    let closures = self.closures;
    let fun = quote_item!(cx,
      #[inline]
      pub fn $name($state_param) -> oak_runtime::ParseState<$stream_ty, $ty>
      {
        $closures
        $body
      }
    ).expect("Quotation of a generated function.");
    if let rust::ItemKind::Fn(a,b,c,d,mut generics,f) = fun.node.clone() {
      let stream_gen = self.grammar.stream_generics();
      generics.params = stream_gen.params;
      generics.where_clause = stream_gen.where_clause;
      let item = rust::Item {
        ident: fun.ident,
        attrs: fun.attrs.clone(),
        id: fun.id,
        node: rust::ItemKind::Fn(a,b,c,d,generics,f),
        vis: fun.vis.clone(),
        span: fun.span,
        tokens: None
      };
      fun.map(|_| item)
    } else { unreachable!() }
  }

  #[allow(unused_imports)] // `quote_tokens` generates a warning.
  fn state_param(&self, state_mut: bool) -> RArg {
    let mut_kw = if state_mut {
      Some(quote_tokens!(self.cx(), mut))
    } else {
      None
    };
    let stream_ty = self.grammar.stream_type();
    quote_arg!(self.cx(), $mut_kw state: oak_runtime::ParseState<$stream_ty, ()>)
  }

  pub fn compile(&mut self, compiler: ExprCompilerFn, idx: usize,
    success: RExpr, failure: RExpr) -> RExpr
  {
    let compiler = compiler(&self.grammar, idx);
    compiler.compile_expr(self, Continuation::new(success, failure))
  }

  pub fn compile_success(&mut self, compiler: ExprCompilerFn, idx: usize,
    success: RExpr, failure: RExpr) -> RExpr
  {
    let expr = self.compile(compiler, idx, success, failure);
    self.num_combinators_compiled += 1;
    expr
  }

  pub fn compile_recognizer_expr(&mut self, idx: usize) -> RExpr {
    Continuation::new(
      quote_expr!(self.cx(), state),
      quote_expr!(self.cx(), state.failure())
    )
    .compile_success(self, recognizer_compiler, idx)
    .unwrap_success()
  }

  pub fn value_constructor<F>(&mut self,
    expr_idx: usize,
    value_ty: RTy,
    value_constructor: F) -> (RExpr, Ident) where
   F: FnOnce(&ExtCtxt, Ident, RExpr) -> RExpr,
  {
    let result_var = self.next_free_var();
    let scope = self.open_scope(expr_idx);
    self.push_mut_ref_fv(result_var, value_ty);
    let span = self.cx().call_site();
    let result_value = tuple_value(self.cx(), span, self.free_variables());
    let body =
      Continuation::new(
        value_constructor(self.cx(), result_var, result_value),
        quote_expr!(self.cx(), state.failure())
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
      let cx = self.cx();
      let closure_name = self.name_factory.next_closure_name(cx);
      let args = self.closure_args();
      let params = self.closure_params();
      continuation.map_success(|success, _| {
        let lambda = cx.lambda_fn_decl(cx.call_site(),
          cx.fn_decl(params, cx.ty_infer(cx.call_site())),
          quote_expr!(cx, $success),
          cx.call_site());
        self.closures.push(quote_stmt!(cx,
          let $closure_name = $lambda;
        ));
        cx.expr_call_ident(cx.call_site(), closure_name, args)
      })
    }
    else {
      continuation
    }
  }

  fn closure_params(&self) -> Vec<RArg> {
    vec![self.state_param(true)]
      .into_iter()
      .chain(self.mut_ref_free_variables
        .iter().cloned()
        .map(|(var, ty)| quote_arg!(self.cx(), $var: &mut $ty)))
      .chain(self.free_variables
        .iter()
        .map(|var| quote_arg!(self.cx(), $var:_)))
      .collect()
  }

  fn closure_args(&self) -> Vec<RExpr> {
    vec![quote_expr!(self.cx(), state)]
      .into_iter()
      .chain(self.mut_ref_free_variables
        .iter().cloned()
        .map(|(var, _)| quote_expr!(self.cx(), &mut $var)))
      .chain(self.free_variables
        .iter()
        .map(|var| quote_expr!(self.cx(), $var)))
      .collect()
  }

  pub fn next_mark_name(&mut self) -> Ident {
    let cx = self.cx();
    self.name_factory.next_mark_name(cx)
  }

  pub fn next_counter_name(&mut self) -> Ident {
    let cx = self.cx();
    self.name_factory.next_counter_name(cx)
  }

  pub fn next_branch_failed_name(&mut self) -> Ident {
    let cx = self.cx();
    self.name_factory.next_branch_failed_name(cx)
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

  pub fn push_mut_ref_fv(&mut self, mut_ref_var: Ident, mut_ref_ty: RTy) {
    self.mut_ref_free_variables.push((mut_ref_var,mut_ref_ty));
  }

  pub fn pop_mut_ref_fv(&mut self) {
    self.mut_ref_free_variables.pop()
      .expect("There is no mut ref free variables.");
  }

  pub fn expr_cardinality(&self, expr_idx: usize) -> usize {
    self.grammar[expr_idx].type_cardinality()
  }

  pub fn expr_span(&self, expr_idx: usize) -> Span {
    self.grammar[expr_idx].span
  }

  pub fn open_scope(&mut self, expr_idx: usize) -> Scope {
    let cx = self.cx();
    let scope = self.save_scope();
    self.num_combinators_compiled = 0;
    self.mut_ref_free_variables = vec![];
    let cardinality = self.expr_cardinality(expr_idx);
    let free_vars = self.name_factory.fresh_vars(cx, cardinality);
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

  pub fn cx(&self) -> &'a ExtCtxt<'b> {
    &self.grammar.cx
  }
}

#[derive(Clone)]
pub struct Scope {
  num_combinators_compiled: usize,
  free_variables: Vec<Ident>,
  mut_ref_free_variables: Vec<(Ident, RTy)>
}

impl Scope {
  fn new(n: usize, fv: Vec<Ident>, mfv: Vec<(Ident, RTy)>) -> Self {
    Scope {
      num_combinators_compiled: n,
      free_variables: fv,
      mut_ref_free_variables: mfv
    }
  }
}
