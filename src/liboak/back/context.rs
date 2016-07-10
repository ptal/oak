
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
use rust::{AstBuilder, TokenTree, Token};

pub struct Context<'a: 'c, 'b: 'a, 'c>
{
  grammar: &'c TGrammar<'a, 'b>,
  closures: Vec<RStmt>,
  name_factory: NameFactory,
  free_variables: Vec<Ident>,
  mut_ref_free_variables: Vec<Ident>,
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
    let closures = self.closures;
    quote_item!(cx,
      #[inline]
      pub fn $name<S>($state_param) -> oak_runtime::ParseState<S, $ty> where
       S: oak_runtime::CharStream
      {
        $closures
        $body
      }
    ).expect("Quotation of a generated function.")
  }

  #[allow(unused_imports)] // `quote_tokens` generates a warning.
  fn state_param(&self, state_mut: bool) -> RArg {
    let mut_kw = if state_mut {
      Some(quote_tokens!(self.cx(), mut))
    } else {
      None
    };
    quote_arg!(self.cx(), $mut_kw state: oak_runtime::ParseState<S, ()>)
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
          quote_block!(cx, { $success }),
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
        .iter()
        .map(|var| quote_arg!(self.cx(), $var: &mut Vec<_>)))
      .chain(self.free_variables
        .iter()
        .map(|var| quote_arg!(self.cx(), $var:_)))
      .collect()
  }

  fn closure_args(&self) -> Vec<RExpr> {
    vec![quote_expr!(self.cx(), state)]
      .into_iter()
      .chain(self.mut_ref_free_variables
        .iter()
        .map(|var| quote_expr!(self.cx(), &mut $var)))
      .chain(self.free_variables
        .iter()
        .map(|var| quote_expr!(self.cx(), $var))) // TODO: compute the type of var.
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

  pub fn next_exit_label(&mut self) -> TokenTree {
    let cx = self.cx();
    let label = self.name_factory.next_exit_label(cx);
    TokenTree::Token(cx.call_site(), Token::Lifetime(label))
  }

  pub fn next_free_var(&mut self) -> Ident {
    self.free_variables.pop().expect("Free variables are all bound.")
  }

  pub fn free_variables(&self) -> Vec<Ident> {
    self.free_variables.clone()
  }

  pub fn open_scope(&mut self, expr_idx: usize, mut_ref_fv: Vec<Ident>) -> Scope {
    let cx = self.cx();
    let scope = self.save_scope();
    self.num_combinators_compiled = 0;
    self.mut_ref_free_variables = mut_ref_fv;
    let cardinality = self.grammar[expr_idx].type_cardinality();
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
  mut_ref_free_variables: Vec<Ident>
}

impl Scope {
  fn new(n: usize, fv: Vec<Ident>, mfv: Vec<Ident>) -> Self {
    Scope {
      num_combinators_compiled: n,
      free_variables: fv,
      mut_ref_free_variables: mfv
    }
  }
}
