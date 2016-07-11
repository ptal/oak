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
    let cx = context.cx();
    // Since we copy the success continuation for each branch, to avoid code explosion, we can extract it into a closure shared by all branches under criterion maintained by the context.
    continuation = context.success_as_closure(continuation);

    let mark = context.next_mark_name();
    let branch_failed = context.next_branch_failed_name();
    context.push_mut_ref_fv(branch_failed, quote_ty!(cx, bool));

    // Each branch of the choice must be compiled in the same variable names environment (they share names of the variables they are building) and with a fresh success continuation size (each branch might create independent success continuation).
    let scope = context.save_scope();

    let mut choices = self.choices.clone();
    let last = choices.pop().unwrap();
    let mut branches: Vec<_> = choices.into_iter()
      .map(|idx| {
        context.restore_scope(scope.clone());
        continuation.compile_and_wrap(context, self.compiler, idx,
          quote_stmt!(cx, $branch_failed = false;))
      })
      .collect();
    // The last branch does not need to assign `false` to the variable `branch_failed`.
    context.restore_scope(scope.clone());
    context.pop_mut_ref_fv();
    let (success, failure) = continuation.unwrap();
    branches.push(context.compile(self.compiler, last, success, failure));

    let mut branches_iter = branches.into_iter();
    let first = branches_iter.next().unwrap();

    let choice = branches_iter
      .rev()
      .fold(quote_expr!(cx, state), |accu, branch|
        quote_expr!(cx,
          if $branch_failed {
            let mut state = state.restore_from_failure($mark.clone());
            let state = $branch;
            $accu
          }
          else { state }
        ));

    quote_expr!(cx, {
      let $mark = state.mark();
      let mut $branch_failed = true;
      let state = $first;
      $choice
    })
  }
}
