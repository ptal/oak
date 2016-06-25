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

use back::compiler::*;

pub struct SequenceCompiler
{
  seq: Vec<usize>,
  compiler: fn(&TGrammar, usize) -> Box<CompileExpr>
}

impl SequenceCompiler
{
  pub fn recognizer(seq: Vec<usize>) -> SequenceCompiler {
    SequenceCompiler {
      seq: seq,
      compiler: recognizer_compiler
    }
  }

  pub fn parser(seq: Vec<usize>) -> SequenceCompiler {
    SequenceCompiler {
      seq: seq,
      compiler: parser_compiler
    }
  }
}

impl CompileExpr for SequenceCompiler
{
  fn compile_expr<'a, 'b, 'c>(&self, context: Context<'a, 'b, 'c>) -> RExpr {
    let mut success = context.success;
    let failure = context.failure;
    let grammar = context.grammar;
    let name_factory = context.name_factory;

    for idx in self.seq.iter().rev().cloned() {
      let compiler = (self.compiler)(grammar, idx);
      success = compiler.compile_expr(Context::new(
        grammar, name_factory, success, failure.clone()));
    }
    success
  }
}
