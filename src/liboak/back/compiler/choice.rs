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
  fn compile_expr<'a, 'b, 'c>(&self,  context: &mut Context<'a, 'b, 'c>,
    mut continuation: Continuation) -> RExpr
  {
    let mark = context.next_mark_name();

    let mut choices = self.choices.clone().into_iter().rev();
    let last = choices.next().unwrap();
    // Since we copy the success continuation for each branch, to avoid code explosion, we can extract it into a closure shared by all branches under criterion maintained by the context.
    continuation = context.success_as_closure(continuation);
    // See Context::save().
    let savepoint = context.save();
    continuation = continuation.compile_failure(context, self.compiler, last);
    // Each branch is compiled and they are nested inside the failure continuation of each other. We must restore the value namespace because each branch has the same type, therefore the success continuation is the same so is the value constructor.
    choices
      .fold(continuation, |continuation, idx| {
        context.restore(savepoint);
        continuation
          .wrap_failure(context, |cx| quote_stmt!(cx,
            state.restore($mark.clone());
          ))
          .compile_failure(context, self.compiler, idx)
      })
      .wrap_failure(context, |cx| quote_stmt!(cx,
        let $mark = state.mark();
      ))
      .unwrap_failure()
  }
}