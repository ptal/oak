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

/// We compile in a bottom-up manner. For example, in `"let" . / "fn" id`, the expression `id` is compiled first, then `"fn"` and then the first branch. We use the trait `CompileExpr` for implementing the compiler of a new combinator. Its method `compile_expr` takes two parameters, the first one is the `context` and the second is the continuation `cont`. The advantage of the bottom-up compilation is that, when compiling a combinator, this combinator can control the future with `cont`. The continuation `cont` contains two attributes: `success` and `failure` representing what has to be done in case of success or failure of the current compiled combinator.

pub mod rtype;
pub mod value;
mod grammar;
mod rule;
mod str_literal;
mod sequence;
mod choice;
mod any_single_char;
mod repeat;
mod optional;
mod syntactic_predicate;
mod character_class;
mod non_terminal;
mod semantic_action;
mod spanned_expr;

pub use back::compiler::grammar::*;
pub use back::context::*;
use back::compiler::str_literal::*;
use back::compiler::sequence::*;
use back::compiler::choice::*;
use back::compiler::any_single_char::*;
use back::compiler::repeat::*;
use back::compiler::optional::*;
use back::compiler::syntactic_predicate::*;
use back::compiler::character_class::*;
use back::compiler::non_terminal::*;
use back::compiler::semantic_action::*;
use back::compiler::spanned_expr::*;

pub enum CompilerKind
{
  Recognizer,
  Parser
}

pub trait CompileExpr
{
  fn compile_expr<'a, 'b, 'c>(&self, context: &mut Context<'a, 'b, 'c>, cont: Continuation) -> RExpr;
}

pub type ExprCompilerFn = fn(&TGrammar, usize) -> Box<CompileExpr>;

pub fn parser_compiler(grammar: &TGrammar, idx: usize) -> Box<CompileExpr> {
  if grammar[idx].ty == Type::Unit {
    recognizer_compiler(grammar, idx)
  }
  else {
    match grammar.expr_by_index(idx) {
      StrLiteral(lit) => Box::new(StrLiteralCompiler::parser(lit)),
      CharacterClass(classes) => Box::new(CharacterClassCompiler::parser(classes)),
      AnySingleChar => Box::new(AnySingleCharCompiler::parser()),
      Sequence(seq) => Box::new(SequenceCompiler::parser(seq)),
      Choice(choices) => Box::new(ChoiceCompiler::parser(choices)),
      ZeroOrOne(expr_idx) => Box::new(OptionalCompiler::parser(expr_idx)),
      ZeroOrMore(expr_idx) => Box::new(RepeatCompiler::parser(expr_idx, 0)),
      OneOrMore(expr_idx) => Box::new(RepeatCompiler::parser(expr_idx, 1)),
      NonTerminalSymbol(id) => Box::new(NonTerminalCompiler::parser(id, idx)),
      SemanticAction(expr_idx, id) => Box::new(SemanticActionCompiler::parser(expr_idx, id, idx)),
      TypeAscription(expr_idx, _) => parser_compiler(grammar, expr_idx),
      SpannedExpr(expr_idx) => Box::new(SpannedExprCompiler::parser(expr_idx)),
      NotPredicate(_)
    | AndPredicate(_) => unreachable!(
        "BUG: Syntactic predicate can not be compiled to parser (they do not generate data)."),
    }
  }
}

pub fn recognizer_compiler(grammar: &TGrammar, idx: usize) -> Box<CompileExpr> {
  match grammar.expr_by_index(idx) {
    StrLiteral(lit) => Box::new(StrLiteralCompiler::recognizer(lit)),
    CharacterClass(classes) => Box::new(CharacterClassCompiler::recognizer(classes)),
    AnySingleChar => Box::new(AnySingleCharCompiler::recognizer()),
    Sequence(seq) => Box::new(SequenceCompiler::recognizer(seq)),
    Choice(choices) => Box::new(ChoiceCompiler::recognizer(choices)),
    ZeroOrOne(expr_idx) => Box::new(OptionalCompiler::recognizer(expr_idx)),
    ZeroOrMore(expr_idx) => Box::new(RepeatCompiler::recognizer(expr_idx, 0)),
    OneOrMore(expr_idx) => Box::new(RepeatCompiler::recognizer(expr_idx, 1)),
    NotPredicate(expr_idx) => Box::new(SyntacticPredicateCompiler::recognizer(expr_idx, Kind::Not)),
    AndPredicate(expr_idx) => Box::new(SyntacticPredicateCompiler::recognizer(expr_idx, Kind::And)),
    NonTerminalSymbol(id) => Box::new(NonTerminalCompiler::recognizer(id)),
    SemanticAction(expr_idx, _) => recognizer_compiler(grammar, expr_idx),
    TypeAscription(expr_idx, _) => recognizer_compiler(grammar, expr_idx),
    SpannedExpr(expr_idx) => recognizer_compiler(grammar, expr_idx),
  }
}
