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
      context.compile(compiler, idx, success, failure))
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
      ))
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
  name_factory: &'c mut NameFactory,
}

impl<'a, 'b, 'c> Context<'a, 'b, 'c>
{
  pub fn new(grammar: &'c TGrammar<'a, 'b>, name_factory: &'c mut NameFactory) -> Self
  {
    Context {
      grammar: grammar,
      name_factory: name_factory
    }
  }

  pub fn compile(&mut self, compiler: ExprCompilerFn, idx: usize,
    success: RExpr, failure: RExpr) -> RExpr
  {
    let compiler = compiler(&self.grammar, idx);
    compiler.compile_expr(self, Continuation::new(success, failure))
  }

  pub fn next_mark_name(&mut self) -> Ident {
    let cx = self.cx();
    self.name_factory.next_mark_name(cx)
  }

  pub fn next_data_name(&mut self) -> Ident {
    self.name_factory.next_data_name()
  }

  pub fn save_namespace(&self) -> Option<Namespace> {
    self.name_factory.save_namespace()
  }

  pub fn restore_namespace(&mut self, namespace: Option<Namespace>) {
    self.name_factory.restore_namespace(namespace)
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
