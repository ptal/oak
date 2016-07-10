
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
  name_factory: NameFactory,
  closure_args: Vec<Vec<Ident>>,
  closures: Vec<RStmt>,
  num_combinators_compiled: usize
}

impl<'a, 'b, 'c> Context<'a, 'b, 'c>
{
  pub fn new(grammar: &'c TGrammar<'a, 'b>) -> Self
  {
    Context {
      grammar: grammar,
      name_factory: NameFactory::new(),
      closure_args: vec![],
      closures: vec![],
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
      println!("create closure");
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
      .chain(
        self.last_closure_args()
          .iter()
          .map(|var| quote_arg!(self.cx(), $var:_)))
      .collect()
  }

  fn closure_args(&self) -> Vec<RExpr> {
    vec![quote_expr!(self.cx(), state)]
      .into_iter()
      .chain(
        self.last_closure_args()
          .iter()
          .map(|var| quote_expr!(self.cx(), $var)))
      .collect()
  }

  pub fn next_mark_name(&mut self) -> Ident {
    let cx = self.cx();
    self.name_factory.next_mark_name(cx)
  }

  pub fn next_exit_label(&mut self) -> TokenTree {
    let cx = self.cx();
    let label = self.name_factory.next_exit_label(cx);
    TokenTree::Token(cx.call_site(), Token::Lifetime(label))
  }

  pub fn next_unbounded_var(&mut self) -> Ident {
    let ident = self.name_factory.next_unbounded_var();
    println!("next_unbounded_var: {}", ident);
    self.push_closure_arg(ident);
    ident
  }

  fn last_closure_args<'d>(&'d self) -> &'d Vec<Ident> {
    self.closure_args.last()
      .expect("Cannot save the context if no scope are opened.")
  }

  // Each branch of the choice must be compiled in a distinct variable names environment (they share names of the variables they are building) and with a fresh success continuation size (each branch might create independent success continuation).
  pub fn save(&self) -> ContextSavepoint {
    ContextSavepoint::new(
      self.name_factory.save_namespace(),
      self.num_combinators_compiled,
      self.last_closure_args().len())
  }

  pub fn restore(&mut self, savepoint: ContextSavepoint) {
    self.name_factory.restore_namespace(savepoint.name_factory);
    self.num_combinators_compiled = savepoint.num_combinators_compiled;
    self.closure_args.last_mut()
      .expect("Cannot restore the context if no scope are opened.")
      .truncate(savepoint.closure_args);
  }

  pub fn push_closure_arg(&mut self, ident: Ident) {
    println!("push_closure_arg: {}", ident);
    self.closure_args.last_mut()
      .expect("Cannot add a closure argument because no scope is currently opened.")
      .push(ident);
  }

  pub fn open_scope(&mut self, expr_idx: usize) -> Vec<Ident> {
    let cx = self.cx();
    self.closure_args.push(vec![]);
    let cardinality = self.grammar[expr_idx].type_cardinality();
    println!("new scope {}", cardinality);
    self.name_factory.open_namespace(cx, cardinality)
  }

  pub fn close_scope(&mut self) {
    self.closure_args.pop();
    self.name_factory.close_namespace();
  }

  pub fn cx(&self) -> &'a ExtCtxt<'b> {
    &self.grammar.cx
  }
}

#[derive(Clone, Copy)]
pub struct ContextSavepoint {
  name_factory: usize,
  num_combinators_compiled: usize,
  closure_args: usize
}

impl ContextSavepoint {
  fn new(name_factory: usize, num_combinators_compiled: usize, closure_args: usize) -> Self {
    ContextSavepoint {
      name_factory: name_factory,
      num_combinators_compiled: num_combinators_compiled,
      closure_args: closure_args
    }
  }
}
