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

pub struct ChoiceCompiler
{
  choices: Vec<usize>,
  compiler: ExprCompilerFn
}

impl ChoiceCompiler
{
  pub fn recognizer(choices: Vec<usize>) -> ChoiceCompiler {
    ChoiceCompiler {
      choices: choices,
      compiler: recognizer_compiler
    }
  }

  pub fn parser(choices: Vec<usize>) -> ChoiceCompiler {
    ChoiceCompiler {
      choices: choices,
      compiler: parser_compiler
    }
  }
}

impl CompileExpr for ChoiceCompiler
{
  fn compile_expr<'a, 'b, 'c>(&self, mut context: Context<'a, 'b, 'c>) -> RExpr {
    // Each branch of the choice must be compiled with the same data namespace.
    let namespace = context.save_namespace();
    let mark_var = context.next_mark_name();

    let mut choices = self.choices.iter().rev().cloned();
    context = context.compile_failure(self.compiler, choices.next().unwrap());

    context = choices.fold(context, |mut context, idx| {
      context.restore_namespace(namespace.clone());
      context.wrap_failure(|cx| quote_stmt!(cx,
        state.restore($mark_var.clone());
      ));
      context.compile_failure(self.compiler, idx)
    });
    context.wrap_failure(|cx| quote_stmt!(cx,
      let $mark_var = state.mark();
    ));
    context.unwrap_failure()
  }
}
