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

pub struct Context<'a: 'c, 'b: 'a, 'c>
{
  pub grammar: &'c TGrammar<'a, 'b>,
  pub name_factory: &'c mut NameFactory,
  pub success: RExpr,
  pub failure: RExpr
}

impl<'a, 'b, 'c> Context<'a, 'b, 'c>
{
  pub fn new(grammar: &'c TGrammar<'a, 'b>,
    name_factory: &'c mut NameFactory,
    success: RExpr, failure: RExpr) -> Self
  {
    Context {
      grammar: grammar,
      name_factory: name_factory,
      success: success,
      failure: failure
    }
  }

  pub fn compile_success(mut self, compiler: ExprCompilerFn, idx: usize) -> Self {
    let compiler = compiler(self.grammar, idx);
    self.success = compiler.compile_expr(Context::new(
      self.grammar, self.name_factory, self.success, self.failure.clone()));
    self
  }

  pub fn compile_failure(mut self, compiler: ExprCompilerFn, idx: usize) -> Self {
    let compiler = compiler(self.grammar, idx);
    self.failure = compiler.compile_expr(Context::new(
      self.grammar, self.name_factory, self.success.clone(), self.failure));
    self
  }

  pub fn wrap_failure<F>(&mut self, f: F) where
   F: FnOnce(&ExtCtxt) -> RStmt
  {
    let stmt = f(self.cx())
      .expect("Statement in wrap_failure.");
    self.failure = {
      let failure = &self.failure;
      quote_expr!(self.cx(), {
        $stmt
        $failure
      })
    }
  }

  pub fn unwrap<F>(self, f: F) -> RExpr where
   F: FnOnce(&ExtCtxt, RExpr, RExpr) -> RExpr
  {
    f(self.cx(), self.success, self.failure)
  }

  pub fn unwrap_failure(self) -> RExpr {
    self.failure
  }

  pub fn unwrap_success(self) -> RExpr {
    self.success
  }

  pub fn next_mark_name(&mut self) -> Ident {
    let cx = self.cx();
    self.name_factory.next_mark_name(cx)
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
  fn compile_expr<'a, 'b, 'c>(&self, context: Context<'a, 'b, 'c>) -> RExpr;
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
