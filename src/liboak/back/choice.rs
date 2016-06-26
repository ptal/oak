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

  fn compile<'a, 'b, 'c>(&self, expr_idx: usize, context: Context<'a, 'b, 'c>) -> RExpr {
    let compiler = (self.compiler)(context.grammar, expr_idx);
    compiler.compile_expr(context)
  }
}

impl CompileExpr for ChoiceCompiler
{
  fn compile_expr<'a, 'b, 'c>(&self, context: Context<'a, 'b, 'c>) -> RExpr {
    let success = context.success;
    let mut failure = context.failure;
    let grammar = context.grammar;
    let name_factory = context.name_factory;
    // Each branch of the choice must be compiled with the same data namespace.
    let namespace = name_factory.save_namespace();
    let mark_var = name_factory.next_mark_name(grammar.cx);

    let mut choices = self.choices.iter().rev().cloned();
    failure = self.compile(choices.next().unwrap(), Context::new(
      grammar, name_factory, success.clone(), failure));

    for idx in choices {
      name_factory.restore_namespace(namespace.clone());
      failure = quote_expr!(grammar.cx, {
        state.restore($mark_var.clone());
        $failure
      });
      failure = self.compile(idx, Context::new(
        grammar, name_factory, success.clone(), failure));
    }
    quote_expr!(grammar.cx, {
      let $mark_var = state.mark();
      $failure
    })
  }
}
