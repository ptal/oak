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

pub use middle::typing::ast::*;
pub use back::name_factory::*;
use back::str_literal::*;
use back::sequence::*;
use back::choice::*;
use back::any_single_char::*;
use back::rtype::*;
use rust::AstBuilder;

pub struct Continuation
{
  success: RExpr,
  failure: RExpr
}

impl Continuation
{
  pub fn new(success: RExpr, failure: RExpr) -> Self {
    Continuation {
      success: success,
      failure: failure
    }
  }

  pub fn compile_success(self, context: &mut Context,
    compiler: ExprCompilerFn, idx: usize) -> Self
  {
    self.map_success(|success, failure|
      context.compile_success(compiler, idx, success, failure))
  }

  pub fn compile_failure(self, context: &mut Context,
    compiler: ExprCompilerFn, idx: usize) -> Self
  {
    self.map_failure(|success, failure|
      context.compile(compiler, idx, success, failure))
  }

  pub fn map_success<F>(mut self, f: F) -> Self where
   F: FnOnce(RExpr, RExpr) -> RExpr
  {
    self.success = f(self.success, self.failure.clone());
    self
  }

  pub fn map_failure<F>(mut self, f: F) -> Self where
   F: FnOnce(RExpr, RExpr) -> RExpr
  {
    self.failure = f(self.success.clone(), self.failure);
    self
  }

  pub fn wrap_failure<F>(self, context: &Context, f: F) -> Self where
   F: FnOnce(&ExtCtxt) -> RStmt
  {
    let stmt = f(context.cx())
      .expect("Statement in wrap_failure.");
    self.map_failure(|_, failure|
      quote_expr!(context.cx(),
        {
          $stmt
          $failure
        }
      )
    )
  }

  pub fn unwrap_failure(self) -> RExpr {
    self.failure
  }

  pub fn unwrap_success(self) -> RExpr {
    self.success
  }
}

pub struct Context<'a: 'c, 'b: 'a, 'c>
{
  grammar: &'c TGrammar<'a, 'b>,
  name_factory: NameFactory,
  closures: Vec<RStmt>,
  success_size: usize
}

impl<'a, 'b, 'c> Context<'a, 'b, 'c>
{
  pub fn new(grammar: &'c TGrammar<'a, 'b>) -> Self
  {
    Context {
      grammar: grammar,
      name_factory: NameFactory::new(),
      closures: vec![],
      success_size: 0
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
    self.success_size += 1;
    expr
  }

  pub fn do_not_duplicate_success(&self) -> bool {
    self.success_size > 0
  }

  pub fn success_as_closure(&mut self, continuation: Continuation) -> Continuation {
    if self.do_not_duplicate_success() {
      self.success_size = 0;
      let cx = self.cx();
      let closure_name = self.name_factory.next_closure_name(cx);
      let mut args = vec![quote_expr!(cx, state)];
      args.extend(self.vars_in_scope_as_args());
      let mut params = vec![self.state_param(true)];
      params.extend(self.vars_in_scope_as_params());
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

  fn vars_in_scope_as_params(&self) -> Vec<RArg> {
    self.name_factory
      .vars_in_scope()
      .into_iter()
      .map(|var| quote_arg!(self.cx(), $var:_))
      .collect()
  }

  fn vars_in_scope_as_args(&self) -> Vec<RExpr> {
    self.name_factory
      .vars_in_scope()
      .into_iter()
      .map(|var| quote_expr!(self.cx(), $var))
      .collect()
  }

  pub fn next_mark_name(&mut self) -> Ident {
    let cx = self.cx();
    self.name_factory.next_mark_name(cx)
  }

  pub fn next_unbounded_var(&mut self) -> Ident {
    self.name_factory.next_unbounded_var()
  }

  // Each branch of the choice must be compiled in a distinct variable names environment (they share names of the variables they are building) and with a fresh success continuation size (each branch might create independent success continuation).
  pub fn save(&self) -> (usize, usize) {
    (self.name_factory.save_namespace(), self.success_size)
  }

  pub fn restore(&mut self, savepoint: (usize, usize)) {
    self.name_factory.restore_namespace(savepoint.0);
    self.success_size = savepoint.1;
  }

  pub fn open_scope(&mut self, expr_idx: usize) -> Vec<Ident> {
    let cx = self.cx();
    let cardinality = self.grammar[expr_idx].type_cardinality();
    self.name_factory.open_namespace(cx, cardinality)
  }

  pub fn close_scope(&mut self) {
    self.name_factory.close_namespace();
  }

  pub fn cx(&self) -> &'a ExtCtxt<'b> {
    &self.grammar.cx
  }
}


pub trait CompileExpr
{
  fn compile_expr<'a, 'b, 'c>(&self, context: &mut Context<'a, 'b, 'c>, cont: Continuation) -> RExpr;
}

pub type ExprCompilerFn = fn(&TGrammar, usize) -> Box<CompileExpr>;

pub fn parser_compiler(grammar: &TGrammar, idx: usize) -> Box<CompileExpr> {
  if grammar[idx].ty.is_unit() {
    recognizer_compiler(grammar, idx)
  }
  else {
    match grammar.expr_by_index(idx) {
      StrLiteral(lit) => Box::new(StrLiteralCompiler::parser(lit)),
      Sequence(seq) => Box::new(SequenceCompiler::parser(seq)),
      AnySingleChar => Box::new(AnySingleCharCompiler::parser()),
      Choice(choices) => Box::new(ChoiceCompiler::parser(choices)),
      _ => unimplemented!()
      // NonTerminalSymbol(id) =>
      // ZeroOrMore(expr) =>
      // OneOrMore(expr) =>
      // Optional(expr) =>
      // NotPredicate(expr) =>
      // AndPredicate(expr) =>
      // CharacterClass(char_class) =>
      // SemanticAction(expr, id) =>
    }
  }
}

pub fn recognizer_compiler(grammar: &TGrammar, idx: usize) -> Box<CompileExpr> {
  match grammar.expr_by_index(idx) {
    StrLiteral(lit) => Box::new(StrLiteralCompiler::recognizer(lit)),
    Sequence(seq) => Box::new(SequenceCompiler::recognizer(seq)),
    AnySingleChar => Box::new(AnySingleCharCompiler::recognizer()),
      Choice(choices) => Box::new(ChoiceCompiler::recognizer(choices)),
    _ => unimplemented!()
    // NonTerminalSymbol(id) =>
    // ZeroOrMore(expr) =>
    // OneOrMore(expr) =>
    // Optional(expr) =>
    // NotPredicate(expr) =>
    // AndPredicate(expr) =>
    // CharacterClass(char_class) =>
    // SemanticAction(expr, id) =>
  }
}
