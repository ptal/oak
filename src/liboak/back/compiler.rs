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
    success: RExpr, failure: RExpr) -> Context<'a, 'b, 'c>
  {
    Context {
      grammar: grammar,
      name_factory: name_factory,
      success: success,
      failure: failure
    }
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
      _ => unimplemented!()
      // AnySingleChar =>
      // NonTerminalSymbol(id) =>
      // Choice(choices) =>
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
    _ => unimplemented!()
    // AnySingleChar =>
    // NonTerminalSymbol(id) =>
    // Choice(choices) =>
    // ZeroOrMore(expr) =>
    // OneOrMore(expr) =>
    // Optional(expr) =>
    // NotPredicate(expr) =>
    // AndPredicate(expr) =>
    // CharacterClass(char_class) =>
    // SemanticAction(expr, id) =>
  }
}
