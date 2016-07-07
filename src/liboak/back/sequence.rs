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
  compiler: ExprCompilerFn
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
  fn compile_expr<'a, 'b, 'c>(&self, context: &mut Context<'a, 'b, 'c>,
    continuation: Continuation) -> RExpr
  {
    self.seq.clone().into_iter()
      .rev()
      .fold(continuation, |continuation, idx|
        continuation.compile_success(context, self.compiler, idx))
      .unwrap_success()
  }
}
